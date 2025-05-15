mod service;

use std::process::{Command, Output};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use std::time::Duration;

pub use service::DockerService;

// De hoofdstructuur voor Docker-tests
pub struct DockerTest {
    // De naam van de Docker image die gebruikt wordt
    image_name: String,
    // Pad naar de map die gemount wordt in de container
    mount_path: Option<PathBuf>,
    // Werkmappad binnen de container
    work_dir: String,
}

impl DockerTest {
    // Creeër een nieuwe DockerTest instantie
    pub fn new(image_name: &str) -> Self {
        DockerTest {
            image_name: image_name.to_string(),
            mount_path: None,
            work_dir: "/scripts",
        }
    }

    // Stel het pad in dat moet worden gemount in de container
    pub fn with_mount(mut self, mount_path: PathBuf) -> Self {
        self.mount_path = Some(mount_path);
        self
    }

    // Stel de werkmap in de container in
    pub fn with_work_dir(mut self, work_dir: &str) -> Self {
        self.work_dir = work_dir.to_string();
        self
    }

    // Voer een bash script uit in de container
    pub fn run_script(&self, script: &str) -> Result<Output, Box<dyn Error>> {
        // Schrijf het script naar een tijdelijk bestand
        let temp_dir = tempfile::tempdir()?;
        let script_path = temp_dir.path().join("test_script.sh");
        
        let mut file = File::create(&script_path)?;
        writeln!(file, "#!/bin/bash")?;
        writeln!(file, "set -e")?;
        writeln!(file, "{}", script)?;
        file.flush()?;
        
        // Maak het script uitvoerbaar
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }
        
        // Bouw de Docker commando argumenten
        let mut args = vec!["run", "--rm"];
        
        // Voeg volume mount toe indien gespecificeerd
        if let Some(mount) = &self.mount_path {
            args.push("-v");
            args.push(&format!("{}:/mnt:ro", mount.display()));
        }
        
        // Mount het script in de container
        args.push("-v");
        args.push(&format!("{}:/tmp/test_script.sh:ro", script_path.display()));
        
        // Stel werkmap in
        args.push("-w");
        args.push(&self.work_dir);
        
        // Voeg de image naam en uit te voeren commando toe
        args.push(&self.image_name);
        args.push("/bin/bash");
        args.push("/tmp/test_script.sh");
        
        // Voer het commando uit
        let output = Command::new("docker").args(&args).output()?;
        
        Ok(output)
    }
    
    // Controleer of Docker beschikbaar is
    pub fn is_docker_available() -> bool {
        Command::new("docker")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    // Bouw de Docker image als die nog niet bestaat
    pub fn build_image(&self, dockerfile_path: &Path) -> Result<(), Box<dyn Error>> {
        // Controleer of de image al bestaat
        let output = Command::new("docker")
            .args(&["images", "-q", &self.image_name])
            .output()?;
        
        if !output.stdout.is_empty() {
            // Image bestaat al
            return Ok(());
        }
        
        // Bouw de image
        let status = Command::new("docker")
            .args(&["build", "-t", &self.image_name, "-f", &dockerfile_path.to_string_lossy(), "."])
            .current_dir(dockerfile_path.parent().unwrap_or_else(|| Path::new(".")))
            .status()?;
        
        if status.success() {
            Ok(())
        } else {
            Err("Failed to build Docker image".into())
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Controleer of Docker beschikbaar is
    if !DockerTest::is_docker_available() {
        eprintln!("Docker is niet beschikbaar. Installeer Docker of start de Docker-daemon.");
        return Err("Docker niet beschikbaar".into());
    }
    
    let dockerfile_path = PathBuf::from("docker_test/Dockerfile");
    
    // Maak een Docker image aan voor gebruik door de service
    let image_name = "heretek-test-alpine";
    let docker_test = DockerTest::new(image_name);
    docker_test.build_image(&dockerfile_path)?;

    println!("Starting Docker service...");
    
    // Start de Docker service
    let (service, result_receiver) = DockerService::new(image_name, None)?;
    
    // Voorbeeld: voer een commando uit via de service
    let command_id = "test-command-1";
    let script = r#"
    echo "Hallo vanuit Docker Alpine container!"
    echo "Huidige map: $(pwd)"
    echo "Bestanden in deze map:"
    ls -la
    echo "Systeem informatie:"
    uname -a
    "#;
    
    // Voeg wat omgevingsvariabelen toe als voorbeeld
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "Dit is een test variabele".to_string());
    
    println!("Executing command {}...", command_id);
    service.execute_command(command_id, script, Some(env_vars), Some(Duration::from_secs(10)))?;
    
    // Wacht op het resultaat
    if let Ok(result) = result_receiver.recv() {
        println!("Command {} completed in {:?}", result.id, result.execution_time);
        
        match result.output {
            Ok(output) => {
                println!("Exit status: {:?}", output.status);
                println!("Stdout:");
                io::stdout().write_all(&output.stdout)?;
                println!("Stderr:");
                io::stderr().write_all(&output.stderr)?;
            },
            Err(e) => {
                println!("Command execution failed: {}", e);
            }
        }
    }
    
    // Nog een commando om te demonstreren dat we meerdere commando's kunnen uitvoeren
    let command_id = "test-command-2";
    let script = "echo 'Dit is een tweede test commando'";
    
    println!("Executing command {}...", command_id);
    service.execute_command(command_id, script, None, None)?;
    
    // Wacht opnieuw op het resultaat
    if let Ok(result) = result_receiver.recv() {
        println!("Command {} completed", result.id);
        if let Ok(output) = result.output {
            io::stdout().write_all(&output.stdout)?;
        }
    }
    
    // Stop de service netjes
    println!("Stopping service...");
    service.stop()?;
    println!("Service stopped");
    
    Ok(())
}