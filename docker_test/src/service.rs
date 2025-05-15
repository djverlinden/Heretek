use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::path::PathBuf;
use std::process::Output;
use std::error::Error;
use std::collections::HashMap;
use std::time::{Duration, Instant};

// Importeer de DockerTest structuur uit main.rs
use crate::DockerTest;

// Bericht types voor de service
pub enum DockerServiceMessage {
    ExecuteCommand { 
        id: String, 
        script: String, 
        env_vars: Option<HashMap<String, String>>,
        timeout: Option<Duration>,
    },
    StopService,
}

// Resultaat van een commando uitvoering
pub struct CommandResult {
    pub id: String,
    pub output: Result<Output, String>,
    pub execution_time: Duration,
}

// De Docker service structuur
pub struct DockerService {
    sender: Sender<DockerServiceMessage>,
    docker_test: Arc<DockerTest>,
    is_running: Arc<Mutex<bool>>,
}

impl DockerService {
    // Start een nieuwe Docker service
    pub fn new(image_name: &str, mount_path: Option<PathBuf>) -> Result<(Self, Receiver<CommandResult>), Box<dyn Error>> {
        // Controleer of Docker beschikbaar is
        if !DockerTest::is_docker_available() {
            return Err("Docker is niet beschikbaar".into());
        }
        
        // Maak een Docker test instantie
        let docker_test = Arc::new(if let Some(path) = mount_path {
            DockerTest::new(image_name).with_mount(path)
        } else {
            DockerTest::new(image_name)
        });
        
        // Kanalen voor communicatie
        let (tx, rx) = mpsc::channel::<DockerServiceMessage>();
        let (result_tx, result_rx) = mpsc::channel::<CommandResult>();
        
        let is_running = Arc::new(Mutex::new(true));
        let is_running_clone = Arc::clone(&is_running);
        let docker_test_clone = Arc::clone(&docker_test);
        
        // Start worker thread
        thread::spawn(move || {
            Self::worker_loop(rx, result_tx, docker_test_clone, is_running_clone);
        });
        
        Ok((
            DockerService {
                sender: tx,
                docker_test,
                is_running,
            },
            result_rx
        ))
    }
    
    // Voer een commando uit via de service
    pub fn execute_command(&self, 
        id: &str, 
        script: &str, 
        env_vars: Option<HashMap<String, String>>,
        timeout: Option<Duration>
    ) -> Result<(), Box<dyn Error>> {
        if !*self.is_running.lock().unwrap() {
            return Err("Service is gestopt".into());
        }
        
        self.sender.send(DockerServiceMessage::ExecuteCommand {
            id: id.to_string(),
            script: script.to_string(),
            env_vars,
            timeout,
        })?;
        
        Ok(())
    }
    
    // Stop de service
    pub fn stop(self) -> Result<(), Box<dyn Error>> {
        *self.is_running.lock().unwrap() = false;
        self.sender.send(DockerServiceMessage::StopService)?;
        Ok(())
    }
    
    // Controleer of de service nog draait
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }
    
    // De worker thread functie
    fn worker_loop(
        receiver: Receiver<DockerServiceMessage>,
        result_sender: Sender<CommandResult>,
        docker_test: Arc<DockerTest>,
        is_running: Arc<Mutex<bool>>
    ) {
        while *is_running.lock().unwrap() {
            match receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(DockerServiceMessage::ExecuteCommand { id, script, env_vars, timeout }) => {
                    let start_time = Instant::now();
                    
                    // Voeg environment variabelen toe aan het script indien aanwezig
                    let script_with_env = if let Some(vars) = env_vars {
                        let env_setup = vars.iter()
                            .map(|(k, v)| format!("export {}=\"{}\"", k, v))
                            .collect::<Vec<_>>()
                            .join("\n");
                        format!("{}\n{}", env_setup, script)
                    } else {
                        script
                    };
                    
                    // Voer het script uit met optionele timeout
                    let result = if let Some(timeout_duration) = timeout {
                        // Create a thread to execute the command
                        let docker_test_clone = Arc::clone(&docker_test);
                        let script_clone = script_with_env.clone();
                        
                        let (tx, rx) = mpsc::channel();
                        
                        let handle = thread::spawn(move || {
                            let result = docker_test_clone.run_script(&script_clone);
                            let _ = tx.send(result);
                        });
                        
                        match rx.recv_timeout(timeout_duration) {
                            Ok(output) => match output {
                                Ok(out) => Ok(out),
                                Err(e) => Err(format!("Fout bij uitvoeren commando: {}", e)),
                            },
                            Err(_) => {
                                // Timeout occurred, thread still running
                                handle.thread().unpark();
                                Err("Commando timeout".to_string())
                            }
                        }
                    } else {
                        // Geen timeout, voer direct uit
                        match docker_test.run_script(&script_with_env) {
                            Ok(out) => Ok(out),
                            Err(e) => Err(format!("Fout bij uitvoeren commando: {}", e)),
                        }
                    };
                    
                    // Stuur het resultaat
                    let execution_time = start_time.elapsed();
                    let _ = result_sender.send(CommandResult {
                        id,
                        output: result,
                        execution_time,
                    });
                }
                Ok(DockerServiceMessage::StopService) => {
                    break;
                }
                Err(_) => {
                    // Timeout bij ontvangen, blijf luisteren
                    continue;
                }
            }
        }
        
        // Service is gestopt
        *is_running.lock().unwrap() = false;
    }
}

// Uitbreiden van DockerTest voor gebruik met de service
impl DockerTest {
    // Functie om een script met environment variabelen uit te voeren
    pub fn run_script_with_env(
        &self, 
        script: &str,
        env_vars: &HashMap<String, String>
    ) -> Result<Output, Box<dyn Error>> {
        let env_prefix = env_vars.iter()
            .map(|(k, v)| format!("export {}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        
        let full_script = format!("{}\n{}", env_prefix, script);
        self.run_script(&full_script)
    }
}