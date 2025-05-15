use std::process::Command;
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

// Importeer onze Docker test module
use container_executor::{DockerTest, DockerService};

// Helper functie om te controleren of Docker beschikbaar is
fn is_docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[test]
fn test_docker_test_creation() {
    let tester = DockerTest::new("test-image");
    assert_eq!(tester.image_name, "test-image");
    assert_eq!(tester.work_dir, "/scripts");
    assert!(tester.mount_path.is_none());
}

#[test]
fn test_docker_test_with_mount() {
    let path = PathBuf::from("/tmp");
    let tester = DockerTest::new("test-image").with_mount(path.clone());
    assert_eq!(tester.mount_path.unwrap(), path);
}

#[test]
fn test_docker_test_with_work_dir() {
    let tester = DockerTest::new("test-image").with_work_dir("/app");
    assert_eq!(tester.work_dir, "/app");
}

#[test]
#[ignore = "Requires Docker to be running"]
fn test_container_available() {
    assert!(DockerTest::is_docker_available());
}

#[test]
#[ignore = "Requires Docker to be running"]
fn test_run_script() {
    // Skip if Docker is not available
    if !is_docker_available() {
        return;
    }
    
    // Bouw de image eerst (dit zou eigenlijk in een setup functie moeten)
    let dockerfile_path = PathBuf::from("../Dockerfile");
    let tester = DockerTest::new("heretek-container-integration");
    
    // Let op: dit kan falen als de Dockerfile niet bestaat op de testlocatie
    match tester.build_image(&dockerfile_path) {
        Ok(_) => {},
        Err(e) => {
            println!("Skipping test, could not build Docker image: {}", e);
            return;
        }
    }
    
    // Voer een eenvoudig script uit
    let script = "echo 'Test succesvol'";
    let result = tester.run_script(script);
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.status.success());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test succesvol"));
}

#[test]
#[ignore = "Requires Docker to be running"]
fn test_docker_service() {
    // Skip if Docker is not available
    if !is_docker_available() {
        return;
    }
    
    // Setup image als nog niet gedaan
    let image_name = "heretek-container-service";
    let dockerfile_path = PathBuf::from("../Dockerfile");
    let docker_test = DockerTest::new(image_name);
    
    match docker_test.build_image(&dockerfile_path) {
        Ok(_) => {},
        Err(e) => {
            println!("Skipping test, could not build Docker image: {}", e);
            return;
        }
    }
    
    // Start de service
    let service_result = DockerService::new(image_name, None);
    assert!(service_result.is_ok());
    
    let (service, result_receiver) = service_result.unwrap();
    
    // Controleer dat de service draait
    assert!(service.is_running());
    
    // Stuur een commando
    let command_id = "test-command";
    let script = "echo 'Service test'";
    
    let result = service.execute_command(command_id, script, None, None);
    assert!(result.is_ok());
    
    // Ontvang het resultaat
    let command_result = result_receiver.recv();
    assert!(command_result.is_ok());
    
    let result = command_result.unwrap();
    assert_eq!(result.id, command_id);
    
    // Controleer de uitvoer
    match result.output {
        Ok(output) => {
            assert!(output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Service test"));
        },
        Err(e) => {
            panic!("Command execution failed: {}", e);
        }
    }
    
    // Test timeout functionaliteit
    let command_id = "test-timeout";
    let script = "sleep 5 && echo 'Dit zou niet zichtbaar moeten zijn'";
    
    let result = service.execute_command(
        command_id, 
        script, 
        None, 
        Some(Duration::from_secs(1))  // 1 seconde timeout
    );
    assert!(result.is_ok());
    
    // Ontvang het resultaat
    let command_result = result_receiver.recv();
    assert!(command_result.is_ok());
    
    let result = command_result.unwrap();
    assert_eq!(result.id, command_id);
    
    // Controleer of de timeout werkte
    match result.output {
        Ok(_) => {
            panic!("Expected timeout but command completed successfully");
        },
        Err(e) => {
            assert!(e.contains("timeout"));
        }
    }
    
    // Stop de service
    let stop_result = service.stop();
    assert!(stop_result.is_ok());
    
    // Geef de thread tijd om te stoppen
    thread::sleep(Duration::from_millis(100));
}