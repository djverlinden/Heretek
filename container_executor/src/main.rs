mod cli;
mod container;
mod tracker;

use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write, stdout};
use std::path::PathBuf;
use std::process::Command;

use clap::Parser;
use cli::{Cli, Commands};
use container::Container;
use tracker::ResourceTracker;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if !check_docker()? {
        return Err("Docker is not available".into());
    }

    match cli.command {
        Commands::Install { template, version } => {
            let image_name = template.get_image_name(&version);
            build_base_image(&image_name, template.as_str())?;
            println!("Template installed successfully!");
            Ok(())
        },
        
        Commands::Cleanup { all, include_base, force } => {
        cleanup_docker(all, include_base, force)?;
        Ok(())
    },
        
        Commands::List { containers, images, detailed } => {
            list_resources(containers, images, detailed)?;
            Ok(())
        },
        
        Commands::Start { template, version, mount, port, name } => {
            // Bepaal image naam
            let image_name = match (template.clone(), version.clone()) {
                (Some(t), Some(ref v)) => t.get_image_name(v),
                (Some(t), None) => t.get_image_name("latest"),
                _ => "heretek-alpine-latest".to_string(),
            };

            if !image_exists(&image_name)? {
                println!("Image {} bestaat niet, installeren...", image_name);
                let template_type = template.map(|t| t.as_str()).unwrap_or("alpine");
                build_base_image(&image_name, template_type)?;
            }
            
            // Start een langlopende container
            start_persistent_container(&image_name, mount, port, name)?;
            Ok(())
        },
        
        Commands::Run { script, template, version, mount, .. } => {
            if !script.exists() {
                return Err(format!("Script not found: {}", script.display()).into());
            }
            
            let mut script_content = String::new();
            File::open(&script)?.read_to_string(&mut script_content)?;
            
            let mut image_name = match (template, version) {
                (Some(t), Some(v)) => t.get_image_name(&v),
                _ => "heretek-alpine-latest".to_string(),
            };

            if !image_exists(&image_name)? {
                let default_image = "heretek-alpine-latest".to_string();
                build_base_image(&default_image, "alpine")?;
                image_name = default_image;
            }
            
            let mut container = Container::new(&image_name);
            if let Some(mount_path) = mount {
                container.with_mount(mount_path);
            }
            
            // Bijhouden van container in de tracker
            if let Some(id) = &container.container_id {
                let mut tracker = ResourceTracker::new();
                tracker.track_container(id);
            }
            
            println!("Starting container with image: {}", image_name);
            container.start()?;
            
            println!("Executing script: {}", script.display());
            let output = container.execute_script(&script_content)?;
            
            std::io::stdout().write_all(&output.stdout)?;
            if !output.status.success() {
                std::io::stderr().write_all(&output.stderr)?;
                return Err("Script execution failed".into());
            }
            
            Ok(())
        }
    }
}

fn check_docker() -> Result<bool, Box<dyn Error>> {
    Ok(Command::new("docker")
        .arg("--version")
        .status()?
        .success())
}

fn image_exists(image_name: &str) -> Result<bool, Box<dyn Error>> {
    Ok(!Command::new("docker")
        .args(&["images", "-q", image_name])
        .output()?
        .stdout
        .is_empty())
}

/// Start een container die blijft draaien (niet automatisch wordt opgeruimd)
fn start_persistent_container(
    image_name: &str,
    mount: Option<PathBuf>,
    port: Option<String>,
    name: Option<String>
) -> Result<(), Box<dyn Error>> {
    // Maak een persistente container (zonder auto-cleanup)
    let mut container = container::Container::new_persistent(image_name);
    
    // Voeg mount toe indien opgegeven
    if let Some(ref mount_path) = mount {
        container.with_mount(mount_path.clone());
    }
    
    // Start de container als een "sleep" process dat blijft draaien
    println!("Starting persistent container with image: {}", image_name);
    
    // Maak een docker run commando
    let mut cmd = Command::new("docker");
    cmd.arg("run")
       .arg("-d") // detached mode
       .arg("--rm"); // verwijder bij stoppen
    
    // Voeg port mapping toe indien opgegeven
    if let Some(ref port_mapping) = port {
        cmd.arg("-p").arg(port_mapping);
    }
    
    // Voeg naam toe indien opgegeven
    if let Some(container_name) = &name {
        cmd.arg("--name").arg(container_name);
    }
    
    // Voeg mount pad toe indien nodig
    if let Some(ref mount_path) = mount {
        cmd.arg("-v")
           .arg(format!("{}:/mnt:ro", mount_path.display()));
    }
    
    // Werkmap en entrypoint
    cmd.arg("-w")
       .arg(&container.work_dir)
       .arg("--entrypoint").arg("/bin/sh")
       .arg(&container.image_name)
       .arg("-c")
       .arg("while true; do sleep 3600; done"); // Blijf draaien
       
    // Voer het commando uit
    let output = cmd.output()?;
    
    if !output.status.success() {
        return Err(format!(
            "Failed to start container: {}", 
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }
    
    let container_id = String::from_utf8(output.stdout)?.trim().to_string();
    
    // Sla container ID op in de container struct
    container.set_container_id(container_id.clone());
    
    // Registreer de container in de tracker
    let mut tracker = ResourceTracker::new();
    tracker.track_container(&container_id);
    
    // Toon info over de gestarte container
    println!("✅ Container gestart!");
    println!("Container ID: {}", container_id);
    
    if let Some(container_name) = name {
        println!("Container naam: {}", container_name);
    }
    
    // Toon hoe de container te benaderen is
    if let Some(ref port_mapping) = port {
        let host_port = port_mapping.split(':').next().unwrap_or(port_mapping);
        println!("Port mapping: {}", port_mapping);
        println!("Benader via: http://localhost:{}", host_port);
    }
    
    println!("\nJe kunt commando's uitvoeren met:");
    println!("  docker exec {} <command>", container_id);
    println!("\nOm te stoppen:");
    println!("  docker stop {}", container_id);
    
    Ok(())
}

fn cleanup_docker(remove_all: bool, include_base: bool, force: bool) -> Result<(), Box<dyn Error>> {
    println!("🧹 Cleaning up Docker resources...");
    
    // Laad de tracker
    let mut tracker = ResourceTracker::new();
    
    // Stop en verwijder geregistreerde containers
    let tracked_containers = tracker.get_containers();
    
    if !tracked_containers.is_empty() {
        println!("Stopping and removing {} tracked containers...", tracked_containers.len());
        
        for container_id in &tracked_containers {
            // Stop de container met force optie indien nodig
            let mut stop_cmd = Command::new("docker");
            stop_cmd.arg("stop");
            
            if force {
                stop_cmd.arg("-f");
            }
            
            stop_cmd.arg(container_id);
            let stop_result = stop_cmd.output()?;
                
            if !stop_result.status.success() {
                println!("Warning: Container {} could not be stopped", container_id);
                if !force {
                    println!("  Tip: Use --force to forcefully stop and remove containers");
                }
                continue;
            }
            
            let rm_result = Command::new("docker")
                .args(["rm", container_id])
                .output()?;
                
            if !rm_result.status.success() {
                println!("Warning: Container {} could not be removed", container_id);
            } else {
                // Verwijder container uit de tracker
                tracker.remove_container(container_id);
            }
        }
    } else {
        println!("No tracked containers found");
    }
    
    // Bepaal welke images te verwijderen
    let tracked_images = tracker.get_images();
    
    // Filter images op basis van include_base parameter
    let images_to_remove: Vec<String> = if include_base {
        tracked_images
    } else {
        // Verwijder alleen images met "heretek-" prefix en niet de base images
        tracked_images.into_iter()
            .filter(|img| img.starts_with("heretek-"))
            .collect()
    };
    
    // Als remove_all=false, vraag dan alleen naar dangling images
    let mut final_images = Vec::new();
    if !remove_all {
        let dangling_output = Command::new("docker")
            .args(&["images", "--filter", "dangling=true", "-q"])
            .output()?;
        
        let dangling_ids = String::from_utf8_lossy(&dangling_output.stdout);
        let dangling_set: HashSet<_> = dangling_ids.trim().split('\n').collect();
        
        // Hou alleen de images over die ook in de dangling lijst staan
        for img in &images_to_remove {
            let inspect_output = Command::new("docker")
                .args(&["inspect", "--format", "{{.Id}}", img])
                .output()?;
                
            if inspect_output.status.success() {
                let id = String::from_utf8_lossy(&inspect_output.stdout).trim().to_string();
                if dangling_set.contains(id.as_str()) {
                    final_images.push(img.clone());
                }
            }
        }
    } else {
        final_images = images_to_remove;
    }
    
    if !final_images.is_empty() {
        println!("Removing {} images...", final_images.len());
        
        for image in &final_images {
            let rmi_result = Command::new("docker")
                .args(["rmi", "-f", image])
                .output()?;
                
            if !rmi_result.status.success() {
                println!("Warning: Image {} could not be removed", image);
                stdout().write_all(&rmi_result.stderr)?;
            } else {
                // Verwijder image uit de tracker
                tracker.remove_image(image);
            }
        }
        
        println!("Removed {} images", final_images.len());
    } else {
        println!("No images to remove");
    }
    
    println!("✅ Cleanup complete!");
    Ok(())
}

/// Toont een lijst van resources die door de container_executor worden bijgehouden
fn list_resources(only_containers: bool, only_images: bool, detailed: bool) -> Result<(), Box<dyn Error>> {
    let tracker = ResourceTracker::new();
    
    // Bepaal wat we moeten tonen (als beide false zijn, toon alles)
    let show_containers = !only_images || only_containers;
    let show_images = !only_containers || only_images;
    
    if show_containers {
        let containers = tracker.get_containers();
        println!("🐳 Tracked Containers ({})", containers.len());
        
        if containers.is_empty() {
            println!("  No tracked containers found");
        } else {
            for (idx, container_id) in containers.iter().enumerate() {
                println!("  {}. {}", idx + 1, container_id);
                
                if detailed {
                    // Voeg gedetailleerde container informatie toe
                    let inspect_output = Command::new("docker")
                        .args(&["inspect", container_id])
                        .output()?;
                        
                    if inspect_output.status.success() {
                        let json_str = String::from_utf8_lossy(&inspect_output.stdout);
                        println!("     Status: {}", get_container_status(&json_str));
                        println!("     Created: {}", get_container_created(&json_str));
                        println!("     Image: {}", get_container_image(&json_str));
                    } else {
                        println!("     (Container niet meer beschikbaar in Docker)");
                    }
                }
            }
        }
        println!();
    }
    
    if show_images {
        let images = tracker.get_images();
        println!("🖼️ Tracked Images ({})", images.len());
        
        if images.is_empty() {
            println!("  No tracked images found");
        } else {
            for (idx, image_name) in images.iter().enumerate() {
                println!("  {}. {}", idx + 1, image_name);
                
                if detailed {
                    // Voeg gedetailleerde image informatie toe
                    let inspect_output = Command::new("docker")
                        .args(&["inspect", image_name])
                        .output()?;
                        
                    if inspect_output.status.success() {
                        let json_str = String::from_utf8_lossy(&inspect_output.stdout);
                        println!("     Created: {}", get_image_created(&json_str));
                        println!("     Size: {}", get_image_size(&json_str));
                        println!("     Tags: {}", get_image_tags(&json_str));
                    } else {
                        println!("     (Image niet meer beschikbaar in Docker)");
                    }
                }
            }
        }
    }
    
    Ok(())
}

// Helper functies voor het parsen van Docker inspect output
fn get_container_status(json_str: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(array) = value.as_array() {
            if let Some(obj) = array.get(0) {
                if let Some(state) = obj.get("State") {
                    if let Some(status) = state.get("Status") {
                        if let Some(status_str) = status.as_str() {
                            return status_str.to_string();
                        }
                    }
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_container_created(json_str: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(array) = value.as_array() {
            if let Some(obj) = array.get(0) {
                if let Some(created) = obj.get("Created") {
                    if let Some(created_str) = created.as_str() {
                        return created_str.to_string();
                    }
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_container_image(json_str: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(array) = value.as_array() {
            if let Some(obj) = array.get(0) {
                if let Some(config) = obj.get("Config") {
                    if let Some(image) = config.get("Image") {
                        if let Some(image_str) = image.as_str() {
                            return image_str.to_string();
                        }
                    }
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_image_created(json_str: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(array) = value.as_array() {
            if let Some(obj) = array.get(0) {
                if let Some(created) = obj.get("Created") {
                    if let Some(created_str) = created.as_str() {
                        return created_str.to_string();
                    }
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_image_size(json_str: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(array) = value.as_array() {
            if let Some(obj) = array.get(0) {
                if let Some(size) = obj.get("Size") {
                    if let Some(size_num) = size.as_u64() {
                        // Convert bytes to human-readable format
                        if size_num < 1024 {
                            return format!("{} B", size_num);
                        } else if size_num < 1024 * 1024 {
                            return format!("{:.2} KB", size_num as f64 / 1024.0);
                        } else if size_num < 1024 * 1024 * 1024 {
                            return format!("{:.2} MB", size_num as f64 / (1024.0 * 1024.0));
                        } else {
                            return format!("{:.2} GB", size_num as f64 / (1024.0 * 1024.0 * 1024.0));
                        }
                    }
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_image_tags(json_str: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(array) = value.as_array() {
            if let Some(obj) = array.get(0) {
                if let Some(repo_tags) = obj.get("RepoTags") {
                    if let Some(tags_array) = repo_tags.as_array() {
                        let tags: Vec<String> = tags_array
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        return tags.join(", ");
                    }
                }
            }
        }
    }
    "None".to_string()
}

fn build_base_image(image_name: &str, template_type: &str) -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    
    // Tracker bijwerken
    let mut tracker = ResourceTracker::new();
    
    let dockerfile_content = match template_type {
        "alpine" => format!(
            "FROM alpine:latest\n\
            RUN apk add --no-cache bash curl openssh-server openssh-client && \\\n\
                mkdir -p /run/sshd && \\\n\
                ssh-keygen -A && \\\n\
                echo 'root:alpine' | chpasswd && \\\n\
                echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config\n\
            WORKDIR /scripts\n\
            EXPOSE 22\n\
            CMD [\"/usr/sbin/sshd\", \"-D\"]"
        ),
        "debian" => format!(
            "FROM debian:stable-slim\n\
            RUN apt-get update && apt-get install -y bash curl openssh-server openssh-client && \\\n\
                mkdir -p /run/sshd && \\\n\
                echo 'root:debian' | chpasswd && \\\n\
                echo 'PermitRootLogin yes' >> /etc/ssh/sshd_config\n\
            WORKDIR /scripts\n\
            EXPOSE 22\n\
            CMD [\"/usr/sbin/sshd\", \"-D\"]"
        ),
        _ => return Err(format!("Unsupported template type: {}", template_type).into())
    };
    
    std::fs::write(&dockerfile_path, dockerfile_content)?;
    
    println!("Building Docker image: {}", image_name);
    let status = Command::new("docker")
        .args(&["build", "-t", image_name, "-f", &dockerfile_path.to_string_lossy(), "."])
        .current_dir(temp_dir.path())
        .status()?;
    
    if !status.success() {
        return Err(format!("Failed to build Docker image: {}", image_name).into());
    }
    
    // Hou het image bij in de tracker
    tracker.track_image(image_name);
    
    Ok(())
}