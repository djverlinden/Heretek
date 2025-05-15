mod service;

use std::process::{Command, Output};
use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};
use std::error::Error;
use std::collections::HashMap;
use std::time::Duration;
use std::env;

pub use service::DockerService;

// Ondersteunde Proxmox template types
pub enum ProxmoxTemplate {
    Alpine,
    Debian,
    Custom(String),
}

// De hoofdstructuur voor Docker-tests
pub struct DockerTest {
    // De naam van de Docker image die gebruikt wordt
    image_name: String,
    // Pad naar de map die gemount wordt in de container
    mount_path: Option<PathBuf>,
    // Werkmappad binnen de container
    work_dir: String,
    // Proxmox template dat wordt gebruikt
    template: Option<ProxmoxTemplate>,
    // Pad naar het template bestand
    template_path: Option<PathBuf>,
}

impl DockerTest {
    // Creeër een nieuwe DockerTest instantie
    pub fn new(image_name: &str) -> Self {
        DockerTest {
            image_name: image_name.to_string(),
            mount_path: None,
            work_dir: "/scripts",
            template: None,
            template_path: None,
        }
    }

    // Creeër een DockerTest instantie met een Proxmox template
    pub fn with_proxmox_template(template: ProxmoxTemplate, template_path: PathBuf) -> Self {
        let image_name = match &template {
            ProxmoxTemplate::Alpine => "proxmox-alpine-test",
            ProxmoxTemplate::Debian => "proxmox-debian-test",
            ProxmoxTemplate::Custom(name) => name,
        };
        
        DockerTest {
            image_name: image_name.to_string(),
            mount_path: None,
            work_dir: "/scripts",
            template: Some(template),
            template_path: Some(template_path),
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

    // Voer een bash script uit in de container via SCP
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
        
        // Start een container in de achtergrond met SSH geïnstalleerd
        let container_id = {
            let output = Command::new("docker")
                .args([
                    "run", 
                    "-d",  // detached mode
                    "--rm",
                    "-v", &format!("{}:/mnt:ro", self.mount_path.as_ref().map_or("", |p| p.to_str().unwrap_or(""))),
                    "--entrypoint", "/bin/sh",
                    &self.image_name,
                    "-c", "apk add --no-cache openssh-server && \
                          mkdir -p /run/sshd && \
                          ssh-keygen -A && \
                          echo 'root:password' | chpasswd && \
                          echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config && \
                          /usr/sbin/sshd && \
                          tail -f /dev/null"  // keep container running
                ])
                .output()?;
            
            if !output.status.success() {
                return Err(format!("Failed to start container: {}", String::from_utf8_lossy(&output.stderr)).into());
            }
            
            String::from_utf8(output.stdout)?.trim().to_string()
        };
        
        // Wacht een moment voor SSH om op te starten
        std::thread::sleep(Duration::from_secs(2));
        
        // Krijg het IP-adres van de container
        let container_ip = {
            let output = Command::new("docker")
                .args(["inspect", "-f", "{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}", &container_id])
                .output()?;
            
            if !output.status.success() {
                // Clean up container
                let _ = Command::new("docker").args(["kill", &container_id]).status();
                return Err(format!("Failed to get container IP: {}", String::from_utf8_lossy(&output.stderr)).into());
            }
            
            String::from_utf8(output.stdout)?.trim().to_string()
        };
        
        // Kopieer het script naar de container met SCP
        {
            // Disable stricthostkeychecking voor dit testgeval
            let scp_args = [
                "-o", "StrictHostKeyChecking=no",
                "-o", "UserKnownHostsFile=/dev/null",
                &script_path.to_string_lossy().to_string(),
                &format!("root@{}:/tmp/script.sh", container_ip)
            ];
            
            let output = Command::new("scp")
                .args(scp_args)
                .env("SSHPASS", "password")
                .output()?;
            
            if !output.status.success() {
                // Clean up container
                let _ = Command::new("docker").args(["kill", &container_id]).status();
                return Err(format!("SCP failed: {}", String::from_utf8_lossy(&output.stderr)).into());
            }
        }
        
        // Voer het script uit via SSH
        let output = Command::new("ssh")
            .args([
                "-o", "StrictHostKeyChecking=no",
                "-o", "UserKnownHostsFile=/dev/null",
                &format!("root@{}", container_ip),
                "chmod +x /tmp/script.sh && cd /scripts && /tmp/script.sh"
            ])
            .env("SSHPASS", "password")
            .output()?;
        
        // Clean up de container
        let _ = Command::new("docker").args(["kill", &container_id]).status();
        
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
        
        // Als we een Proxmox template gebruiken, maak dan een image op basis daarvan
        if let Some(template_path) = &self.template_path {
            return self.build_from_proxmox_template(template_path);
        }
        
        // Standaard image bouwen
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
    
    // Bouw een Docker image van een Proxmox template
    fn build_from_proxmox_template(&self, template_path: &Path) -> Result<(), Box<dyn Error>> {
        println!("Building Docker image from Proxmox template: {}", template_path.display());
        
        // Tijdelijke map aanmaken voor het bouwen
        let temp_dir = tempfile::tempdir()?;
        let rootfs_dir = temp_dir.path().join("rootfs");
        fs::create_dir_all(&rootfs_dir)?;
        
        // Extract het template (afhankelijk van het type)
        let template_extension = template_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        
        if template_extension == "tar" || template_extension == "gz" || template_extension == "xz" {
            println!("Extracting template archive...");
            
            let extract_cmd = match template_extension {
                "tar" => Command::new("tar")
                    .args(&["xf", &template_path.to_string_lossy(), "-C", &rootfs_dir.to_string_lossy()])
                    .status()?,
                "gz" => Command::new("tar")
                    .args(&["xzf", &template_path.to_string_lossy(), "-C", &rootfs_dir.to_string_lossy()])
                    .status()?,
                "xz" => Command::new("tar")
                    .args(&["xJf", &template_path.to_string_lossy(), "-C", &rootfs_dir.to_string_lossy()])
                    .status()?,
                _ => return Err(format!("Unsupported template format: {}", template_extension).into()),
            };
            
            if !extract_cmd.success() {
                return Err(format!("Failed to extract template archive").into());
            }
        } else {
            return Err(format!("Unsupported template format: {}", template_extension).into());
        }
        
        // Maak een minimale Dockerfile aan die de rootfs gebruikt
        let dockerfile_path = temp_dir.path().join("Dockerfile");
        let mut dockerfile = File::create(&dockerfile_path)?;
        
        writeln!(dockerfile, "FROM scratch")?;
        writeln!(dockerfile, "ADD rootfs /\n")?;
        
        // Voeg de nodige pakketten toe voor SSH (afhankelijk van de template)
        match &self.template {
            Some(ProxmoxTemplate::Alpine) => {
                writeln!(dockerfile, "RUN apk update && \\")?;
                writeln!(dockerfile, "    apk add --no-cache bash openssh-server openssh-client && \\")?;
                writeln!(dockerfile, "    mkdir -p /run/sshd && \\")?;
                writeln!(dockerfile, "    ssh-keygen -A && \\")?;
                writeln!(dockerfile, "    echo 'root:alpine' | chpasswd && \\")?;
                writeln!(dockerfile, "    echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config")?;
            },
            Some(ProxmoxTemplate::Debian) => {
                writeln!(dockerfile, "RUN apt-get update && \\")?;
                writeln!(dockerfile, "    apt-get install -y openssh-server openssh-client && \\")?;
                writeln!(dockerfile, "    mkdir -p /run/sshd && \\")?;
                writeln!(dockerfile, "    echo 'root:debian' | chpasswd && \\")?;
                writeln!(dockerfile, "    echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config")?;
            },
            _ => {
                // Generieke setup voor andere templates
                writeln!(dockerfile, "RUN if command -v apt-get; then \\")?;
                writeln!(dockerfile, "        apt-get update && apt-get install -y openssh-server; \\")?;
                writeln!(dockerfile, "    elif command -v apk; then \\")?;
                writeln!(dockerfile, "        apk add --no-cache openssh-server; \\")?;
                writeln!(dockerfile, "    elif command -v yum; then \\")?;
                writeln!(dockerfile, "        yum install -y openssh-server; \\")?;
                writeln!(dockerfile, "    fi && \\")?;
                writeln!(dockerfile, "    mkdir -p /run/sshd && \\")?;
                writeln!(dockerfile, "    echo 'root:password' | chpasswd && \\")?;
                writeln!(dockerfile, "    echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config")?;
            }
        }
        
        writeln!(dockerfile, "\nWORKDIR {}", self.work_dir)?;
        writeln!(dockerfile, "EXPOSE 22\n")?;
        writeln!(dockerfile, "CMD [\"/usr/sbin/sshd\", \"-D\"]")?;
        
        dockerfile.flush()?;
        
        // Bouw de Docker image
        println!("Building Docker image from Proxmox template...");
        let build_cmd = Command::new("docker")
            .args(&["build", "-t", &self.image_name, "-f", &dockerfile_path.to_string_lossy(), "."])
            .current_dir(temp_dir.path())
            .status()?;
        
        if !build_cmd.success() {
            return Err("Failed to build Docker image from Proxmox template".into());
        }
        
        println!("Successfully built Docker image {} from Proxmox template", self.image_name);
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Controleer of Docker beschikbaar is
    if !DockerTest::is_docker_available() {
        eprintln!("Docker is niet beschikbaar. Installeer Docker of start de Docker-daemon.");
        return Err("Docker niet beschikbaar".into());
    }
    
    // Voorbeeld van normale Docker image
    let dockerfile_path = PathBuf::from("container_executor/Dockerfile");
    let image_name = "heretek-test-alpine";
    let docker_test = DockerTest::new(image_name);
    docker_test.build_image(&dockerfile_path)?;

    // Voorbeeld van het gebruik van een Proxmox template (als het bestaat)
    let proxmox_template_path = PathBuf::from("alpine-3.12-default_2020-04-29_amd64.tar.xz");
    if proxmox_template_path.exists() {
        println!("Proxmox template gevonden, bouw een image...");
        let proxmox_docker_test = DockerTest::with_proxmox_template(
            ProxmoxTemplate::Alpine, 
            proxmox_template_path
        );
        proxmox_docker_test.build_image(&PathBuf::from("dummy"))?; // Het pad wordt genegeerd voor templates
        
        // In een echte toepassing zou je deze image kunnen gebruiken
        // Voor dit voorbeeld blijven we bij de standaard image
    } else {
        println!("Geen Proxmox template gevonden op het standaard pad.");
        println!("Standaard Docker image wordt gebruikt.");
    }

    println!("Starting Docker service...");
    
    // Start de Docker service
    let (service, result_receiver) = DockerService::new(image_name, None)?;
    
    println!("Zorg ervoor dat 'sshpass' geïnstalleerd is (op macOS: brew install hudochenkov/sshpass/sshpass).");
    println!("Zorg ervoor dat 'sshpass' en 'scp' geïnstalleerd zijn voor SCP-functionaliteit.");
    
    // Voorbeeld: voer een commando uit via de service
    let command_id = "test-command-1";
    let script = r#"
    echo "Hallo vanuit Docker Alpine container via SCP!"
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