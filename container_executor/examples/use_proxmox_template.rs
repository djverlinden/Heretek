use std::path::PathBuf;
use std::error::Error;
use std::time::Duration;
use std::collections::HashMap;

use container_executor::{DockerTest, DockerService, ProxmoxTemplate};

fn main() -> Result<(), Box<dyn Error>> {
    // Controleer of Docker beschikbaar is
    if !DockerTest::is_docker_available() {
        eprintln!("Docker is niet beschikbaar. Installeer Docker of start de Docker-daemon.");
        return Err("Docker niet beschikbaar".into());
    }
    
    // Pad naar je Proxmox template
    let template_path = PathBuf::from("alpine-3.12-default_2020-04-29_amd64.tar.xz");
    
    if !template_path.exists() {
        eprintln!("Proxmox template bestand niet gevonden: {}", template_path.display());
        eprintln!("Plaats een geldig Proxmox template bestand op deze locatie.");
        return Err("Template bestand niet gevonden".into());
    }
    
    println!("Bouwen van Docker image vanuit Proxmox template: {}", template_path.display());
    
    // Maak een DockerTest instantie met het Proxmox template
    let docker_test = DockerTest::with_proxmox_template(
        ProxmoxTemplate::Alpine, 
        template_path
    );
    
    // Bouw de Docker image vanuit het template
    docker_test.build_image(&PathBuf::from("dummy"))?; // Het pad wordt genegeerd voor templates
    
    println!("Docker image succesvol gebouwd. Nu een test script uitvoeren...");
    
    // Voorbeeld script om uit te voeren in de container
    let script = r#"
    echo "== Proxmox Template Test =="
    echo "Draait in container op: $(date)"
    echo "Systeem info:"
    uname -a
    
    # Check welke Linux distributie we hebben
    if [ -f /etc/os-release ]; then
        echo "Distributie informatie:"
        cat /etc/os-release
    fi
    
    # Test commando's beschikbaar zijn
    echo "Bash versie: $(bash --version | head -n 1)"
    
    # Maak een testbestand aan
    echo "Dit is een test" > /tmp/test.txt
    echo "Testbestand aangemaakt:"
    cat /tmp/test.txt
    
    echo "== Test voltooid =="
    "#;
    
    // Voer het script uit
    let output = docker_test.run_script(script)?;
    
    println!("Script exit status: {:?}", output.status);
    println!("Script uitvoer:");
    std::io::stdout().write_all(&output.stdout)?;
    
    // Alternatief: gebruik de service API om commando's uit te voeren
    println!("\n\nTesten met Docker Service API:");
    
    // Start een Docker service met het Proxmox template
    let (service, result_receiver) = DockerService::with_proxmox_template(
        ProxmoxTemplate::Alpine,
        template_path,
        None  // Geen mount pad
    )?;
    
    // Voer meerdere commando's uit via de service
    let command_id = "proxmox-test-1";
    let script = "echo 'Test van de Docker Service API met Proxmox template'";
    
    println!("Uitvoeren van commando {}...", command_id);
    service.execute_command(command_id, script, None, None)?;
    
    // Wacht op het resultaat
    if let Ok(result) = result_receiver.recv() {
        println!("Commando {} voltooid", result.id);
        if let Ok(output) = result.output {
            std::io::stdout().write_all(&output.stdout)?;
        }
    }
    
    // Stop de service
    println!("Service stoppen...");
    service.stop()?;
    
    println!("Test voltooid");
    Ok(())
}