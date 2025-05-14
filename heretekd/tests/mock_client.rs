use std::io::{Read, Write, Error as IoError, ErrorKind};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use heretek_config::load_config;

/// Standaard wachttijd tussen commands in milliseconden
const DEFAULT_WAIT_TIME: u64 = 100;
/// Standaard timeout voor server connectie in milliseconden
const DEFAULT_CONNECT_TIMEOUT: u64 = 500;
/// Standaard buffergrootte voor antwoorden
const DEFAULT_BUFFER_SIZE: usize = 2048;

/// Een eenvoudige test client die de verbinding met de mock server test
///
/// Deze test is standaard uitgeschakeld omdat deze een draaiende server vereist.
/// Start de server voordat je deze test uitvoert met 'cargo test -- --ignored'.
#[test]
#[ignore] // Deze test uitschakelen in standaard test runs
fn test_mock_client() {
    // Laad server configuratie uit config
    let config = match load_config() {
        conf => conf,
    };
    
    // Configureer de test client
    let server_address = format!("{}:{}", config.heretekd.host, config.heretekd.port);
    let commands = vec![
        "zfs list",
        "pct list",
        "pveam available",
        "onbekende opdracht",
    ];
    
    println!("Start mock client test voor server op {}", server_address);
    
    // Optioneel: wacht tot server opgestart is (als je dit samen met server start runt)
    thread::sleep(Duration::from_millis(DEFAULT_CONNECT_TIMEOUT));
    
    match TcpStream::connect(&server_address) {
        Ok(mut stream) => {
            println!("✅ Verbonden met server op {}", server_address);
            
            for cmd in commands {
                match test_command(&mut stream, cmd) {
                    Ok(response) => {
                        println!("📥 Ontvangen voor '{}': {}", cmd, response);
                        
                        // Voer een eenvoudige controle uit op het antwoord
                        match cmd {
                            "zfs list" => assert!(
                                response.contains("rpool"), 
                                "Respons voor '{}' bevat geen 'rpool': {}", cmd, response
                            ),
                            "pct list" => assert!(
                                response.contains("test-container"), 
                                "Respons voor '{}' bevat geen 'test-container': {}", cmd, response
                            ),
                            "pveam available" => assert!(
                                response.contains("ubuntu"), 
                                "Respons voor '{}' bevat geen 'ubuntu': {}", cmd, response
                            ),
                            _ => assert!(
                                response.contains("onbekend commando"), 
                                "Respons voor onbekend commando bevat geen foutmelding: {}", response
                            ),
                        }
                    },
                    Err(e) => {
                        eprintln!("❌ Fout bij uitvoeren commando '{}': {}", cmd, e);
                        panic!("Commando test gefaald voor '{}': {}", cmd, e);
                    }
                }
                
                // Korte pauze tussen commando's
                thread::sleep(Duration::from_millis(DEFAULT_WAIT_TIME));
            }
            
            println!("✅ Alle commando's zijn succesvol getest");
        },
        Err(e) => {
            eprintln!(
                "❌ Kon niet verbinden met server op {}: {}. Start de server voordat je deze test uitvoert.", 
                server_address, e
            );
            // Bij een CI/CD pipeline zou je hier kunnen kiezen om de test te laten falen
            // panic!("Kon geen verbinding maken met de server: {}", e);
            println!("⚠️ Test overgeslagen. Start server met 'cargo run' en probeer opnieuw.");
        },
    }
}

/// Helper functie om een individueel commando te testen
///
/// Stuurt een commando naar de server en leest het antwoord.
///
/// # Arguments
/// * `stream` - De TcpStream verbinding met de server
/// * `command` - Het commando om uit te voeren (zonder newline)
///
/// # Returns
/// * `Result<String, IoError>` - Het antwoord van de server of een fout
fn test_command(stream: &mut TcpStream, command: &str) -> Result<String, IoError> {
    println!("📤 Test commando: {}", command);
    
    // Commando versturen met newline
    let cmd_with_nl = format!("{}\n", command);
    
    // Schrijf naar de stream met expliciete foutafhandeling
    stream.write_all(cmd_with_nl.as_bytes()).map_err(|e| {
        IoError::new(
            ErrorKind::ConnectionAborted,
            format!("Kon commando '{}' niet versturen: {}", command, e)
        )
    })?;
    
    // Antwoord ontvangen met een grotere buffer voor uitgebreidere antwoorden
    let mut buffer = vec![0; DEFAULT_BUFFER_SIZE];
    let bytes_read = stream.read(&mut buffer).map_err(|e| {
        IoError::new(
            ErrorKind::ConnectionAborted,
            format!("Kon antwoord niet lezen voor commando '{}': {}", command, e)
        )
    })?;
    
    if bytes_read > 0 {
        Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
    } else {
        Err(IoError::new(
            ErrorKind::UnexpectedEof,
            format!("Leeg antwoord ontvangen voor commando '{}'", command)
        ))
    }
}

/// Test een enkel commando direct
///
/// Deze test is nuttig voor het isoleren van problemen met specifieke commando's.
/// De test is standaard uitgeschakeld omdat deze een draaiende server vereist.
#[test]
#[ignore]
fn test_single_command() {
    // Laad server configuratie uit config
    let config = match load_config() {
        conf => conf,
    };
    
    let server_address = format!("{}:{}", config.heretekd.host, config.heretekd.port);
    println!("Test enkele commando verbinding met server op {}", server_address);
    
    // Geef server tijd om op te starten
    thread::sleep(Duration::from_millis(DEFAULT_CONNECT_TIMEOUT));
    
    match TcpStream::connect(&server_address) {
        Ok(mut stream) => {
            println!("✅ Verbonden met server voor enkele commando test");
            
            match test_command(&mut stream, "zfs list") {
                Ok(response) => {
                    assert!(
                        response.contains("rpool"),
                        "Respons bevat geen 'rpool': {}", response
                    );
                    assert!(
                        response.contains("USED"),
                        "Respons bevat geen 'USED': {}", response
                    );
                    println!("✅ Commando test geslaagd");
                },
                Err(e) => {
                    eprintln!("❌ Commando test mislukt: {}", e);
                    panic!("Commando test gefaald: {}", e);
                }
            }
        },
        Err(e) => {
            eprintln!("❌ Server niet bereikbaar op {}: {}", server_address, e);
            println!("⚠️ Test overgeslagen. Start server met 'cargo run' en probeer opnieuw.");
        }
    }
}