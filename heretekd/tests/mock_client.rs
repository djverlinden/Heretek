use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// Een eenvoudige test client die de verbinding met de mock server test
#[test]
#[ignore] // Deze test uitschakelen in standaard test runs
fn test_mock_client() {
    // Configureer de test client
    let server_address = "127.0.0.1:2222"; // Default test address
    let commands = vec![
        "zfs list",
        "pct list",
        "pveam available",
        "onbekende opdracht",
    ];
    
    // Optioneel: wacht tot server opgestart is (als je dit samen met server start runt)
    thread::sleep(Duration::from_millis(500));
    
    match TcpStream::connect(server_address) {
        Ok(mut stream) => {
            println!("Verbonden met server op {}", server_address);
            
            for cmd in commands {
                match test_command(&mut stream, cmd) {
                    Ok(response) => {
                        println!("Ontvangen voor {}: {}", cmd, response);
                        
                        // Voer een eenvoudige controle uit op het antwoord
                        match cmd {
                            "zfs list" => assert!(response.contains("rpool")),
                            "pct list" => assert!(response.contains("test-container")),
                            "pveam available" => assert!(response.contains("ubuntu")),
                            _ => assert!(response.contains("onbekend commando")),
                        }
                    },
                    Err(e) => println!("Fout bij uitvoeren commando '{}': {}", cmd, e),
                }
                
                // Korte pauze tussen commando's
                thread::sleep(Duration::from_millis(100));
            }
        },
        Err(e) => println!(
            "Kon niet verbinden met server op {}: {}. Test overgeslagen.",
            server_address, e
        ),
    }
}

/// Helper functie om een individueel commando te testen
fn test_command(stream: &mut TcpStream, command: &str) -> Result<String, std::io::Error> {
    println!("Test commando: {}", command);
    
    // Commando versturen met newline
    let cmd_with_nl = format!("{}\n", command);
    stream.write_all(cmd_with_nl.as_bytes())?;
    
    // Antwoord ontvangen
    let mut buffer = vec![0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    
    if bytes_read > 0 {
        Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
    } else {
        Ok("".to_string())
    }
}

/// Test een enkel commando direct
#[test]
#[ignore]
fn test_single_command() {
    let server_address = "127.0.0.1:2222";
    thread::sleep(Duration::from_millis(100));
    
    if let Ok(mut stream) = TcpStream::connect(server_address) {
        if let Ok(response) = test_command(&mut stream, "zfs list") {
            assert!(response.contains("rpool"));
            assert!(response.contains("USED"));
        }
    } else {
        println!("Server niet bereikbaar, test overgeslagen");
    }
}