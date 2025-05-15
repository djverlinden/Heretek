use std::path::PathBuf;
use std::process::{Command, Output};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use crate::tracker::ResourceTracker;

pub struct Container {
    pub image_name: String,
    pub mount_path: Option<PathBuf>,
    pub container_id: Option<String>,
    #[allow(dead_code)]
    pub work_dir: String,
    pub auto_cleanup: bool,
}

impl Container {
    pub fn new(image_name: &str) -> Self {
        Self {
            image_name: image_name.to_string(),
            mount_path: None,
            container_id: None,
            work_dir: "/scripts".to_string(),
            auto_cleanup: true,
        }
    }
    
    /// Maak een nieuwe container die niet automatisch wordt opgeruimd
    pub fn new_persistent(image_name: &str) -> Self {
        Self {
            image_name: image_name.to_string(),
            mount_path: None,
            container_id: None,
            work_dir: "/scripts".to_string(),
            auto_cleanup: false,
        }
    }
    
    /// Update container ID
    pub fn set_container_id(&mut self, id: String) {
        self.container_id = Some(id);
    }

    pub fn with_mount(&mut self, mount_path: PathBuf) {
        self.mount_path = Some(mount_path);
    }
    
    #[allow(dead_code)]
    pub fn with_work_dir(&mut self, work_dir: &str) {
        self.work_dir = work_dir.to_string();
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::new("docker");
        cmd.arg("run")
           .arg("-d")
           .arg("--rm");

        if let Some(mount) = &self.mount_path {
            cmd.arg("-v")
               .arg(format!("{}:/mnt:ro", mount.display()));
        }

        cmd.arg("-w")
           .arg("/scripts")
           .arg(&self.image_name);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(format!("Failed to start container: {}", 
                String::from_utf8_lossy(&output.stderr)).into());
        }

        let container_id = String::from_utf8(output.stdout)?.trim().to_string();
        self.container_id = Some(container_id.clone());
        
        // Bijhouden van container in tracker
        let mut tracker = ResourceTracker::new();
        tracker.track_container(&container_id);
        
        Ok(())
    }

    pub fn execute_script(&self, script: &str) -> Result<Output, Box<dyn Error>> {
        let container_id = self.container_id.as_ref()
            .ok_or("Container not started")?;

        let temp_dir = tempfile::tempdir()?;
        let script_path = temp_dir.path().join("script.sh");
        let mut file = File::create(&script_path)?;
        writeln!(file, "#!/bin/bash\nset -e\n{}", script)?;
        file.flush()?;

        Command::new("docker")
            .arg("cp")
            .arg(&script_path)
            .arg(format!("{}:/tmp/script.sh", container_id))
            .status()?;

        Command::new("docker")
            .arg("exec")
            .arg(container_id)
            .arg("/bin/bash")
            .arg("-c")
            .arg("chmod +x /tmp/script.sh && cd /scripts && /tmp/script.sh")
            .output()
            .map_err(|e| e.into())
    }
    
    #[allow(dead_code)]
    pub fn get_ip(&self) -> Result<String, Box<dyn Error>> {
        let container_id = self.container_id.as_ref()
            .ok_or("Container not started")?;
            
        let output = Command::new("docker")
            .arg("inspect")
            .arg("-f")
            .arg("{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}")
            .arg(container_id)
            .output()?;
            
        if !output.status.success() {
            return Err(format!("Failed to get IP address: {}", 
                String::from_utf8_lossy(&output.stderr)).into());
        }
        
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }
    
    // Docker test API (voor compatibiliteit met testen)
    #[allow(dead_code)]
    pub fn is_docker_available() -> bool {
        Command::new("docker")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    #[allow(dead_code)]
    pub fn build_image(&self, dockerfile_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let status = Command::new("docker")
            .args(["build", "-t", &self.image_name, "-f"])
            .arg(dockerfile_path)
            .arg(".")
            .status()?;

        if !status.success() {
            return Err(format!("Failed to build Docker image: {}", self.image_name).into());
        }

        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn run_script(&self, script: &str) -> Result<Output, String> {
        let temp_dir = match tempfile::tempdir() {
            Ok(dir) => dir,
            Err(e) => return Err(format!("Failed to create temp directory: {}", e)),
        };
        let script_path = temp_dir.path().join("script.sh");
        let mut file = match File::create(&script_path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Failed to create script file: {}", e)),
        };
        
        if let Err(e) = writeln!(file, "#!/bin/bash\nset -e\n{}", script) {
            return Err(format!("Failed to write script: {}", e));
        }
        
        if let Err(e) = file.flush() {
            return Err(format!("Failed to flush script file: {}", e));
        }

        let mut cmd = Command::new("docker");
        cmd.arg("run")
           .arg("--rm");

        if let Some(mount) = &self.mount_path {
            cmd.arg("-v")
               .arg(format!("{}:/mnt:ro", mount.display()));
        }

        cmd.arg("-w")
           .arg(&self.work_dir)
           .arg(&self.image_name)
           .arg("/bin/bash")
           .arg("-c")
           .arg(format!("{}", script));

        cmd.output()
           .map_err(|e| format!("Failed to execute command: {}", e))
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        // Alleen opruimen als we een auto_cleanup container hebben
        if self.auto_cleanup {
            if let Some(id) = &self.container_id {
                let result = Command::new("docker")
                    .arg("stop")
                    .arg(id)
                    .status();
                    
                // Als container succesvol is gestopt, verwijder uit tracker
                if result.is_ok() && result.unwrap().success() {
                    let mut tracker = ResourceTracker::new();
                    tracker.remove_container(id);
                }
            }
        }
    }
}