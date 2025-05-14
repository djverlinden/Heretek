use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;


const TEST_HOST: &str = "127.0.0.1";
const TEST_PORT: u16 = 2223;
const RETRY_DELAY: Duration = Duration::from_millis(100);
const MAX_RETRIES: u32 = 5;

fn wait_for_server() -> Option<TcpStream> {
    for _ in 0..MAX_RETRIES {
        match TcpStream::connect((TEST_HOST, TEST_PORT)) {
            Ok(stream) => return Some(stream),
            Err(_) => thread::sleep(RETRY_DELAY),
        }
    }
    None
}

/// Test helper functie om commando's naar de server te sturen
fn send_command(stream: &mut TcpStream, command: &str) -> Result<String, std::io::Error> {
    let cmd_with_nl = format!("{}\n", command);
    stream.write_all(cmd_with_nl.as_bytes())?;
    
    let mut buffer = vec![0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
}

#[test]
#[ignore]
fn test_server_basic_commands() {
    // Wacht tot server beschikbaar is
    let mut stream = wait_for_server()
        .expect("Server niet beschikbaar na meerdere pogingen");

    // Test basis commando's
    let test_cases = vec![
        ("pct list", "100 debian10 running"),
        ("zfs list", "rpool 1G 99G"),
        ("unknown", "Onbekend commando"),
    ];

    for (command, expected) in test_cases {
        let response = send_command(&mut stream, command)
            .unwrap_or_else(|e| panic!("Fout bij versturen commando '{}': {}", command, e));
        
        assert!(
            response.contains(expected),
            "Voor commando '{}': verwachtte '{}', kreeg '{}'",
            command,
            expected,
            response
        );
    }
}

#[test]
#[ignore]
fn test_server_concurrent_connections() {
    let mut handles = vec![];
    
    // Start meerdere clients tegelijk
    for i in 0..3 {
        let handle = std::thread::spawn(move || {
            if let Some(mut stream) = wait_for_server() {
                let cmd = "pct list";
                let response = send_command(&mut stream, cmd)
                    .unwrap_or_else(|e| panic!("Client {} fout: {}", i, e));
                
                assert!(
                    response.contains("100 debian10 running"),
                    "Client {} kreeg onverwachte response: {}", 
                    i, 
                    response
                );
            }
        });
        handles.push(handle);
    }

    // Wacht tot alle clients klaar zijn
    for handle in handles {
        handle.join().expect("Client task failed");
    }
}

#[test]
#[ignore]
fn test_server_long_session() {
    if let Some(mut stream) = wait_for_server() {
        // Test een langere sessie met meerdere commando's
        let commands = vec!["pct list", "zfs list", "pct list"];
        
        for cmd in commands {
            let response = send_command(&mut stream, cmd)
                .unwrap_or_else(|e| panic!("Fout tijdens lange sessie: {}", e));
            
            assert!(
                !response.is_empty(),
                "Lege response ontvangen voor commando: {}", 
                cmd
            );
            
            // Korte pauze tussen commando's
            thread::sleep(Duration::from_millis(100));
        }
    }
}

#[test]
#[ignore]
fn test_server_invalid_commands() {
    if let Some(mut stream) = wait_for_server() {
        let invalid_commands = vec![
            "",
            "invalid",
            "unknown_command",
            "pct",
            "zfs",
        ];

        for cmd in invalid_commands {
            let response = send_command(&mut stream, cmd)
                .unwrap_or_else(|e| panic!("Fout bij invalid commando test: {}", e));
            
            assert!(
                response.contains("Onbekend commando"),
                "Verwachtte foutmelding voor '{}', kreeg: '{}'",
                cmd,
                response
            );
        }
    }
}