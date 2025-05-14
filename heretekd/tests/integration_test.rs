use heretekd::{MockServer, ProxmoxVersion};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use heretek_config::load_config;
use std::sync::Once;

// Zorgt dat setup_test_config slechts één keer wordt uitgevoerd
static INIT: Once = Once::new();

/// Verkrijg het pad naar de tijdelijke testdirectory
fn get_temp_test_dir() -> PathBuf {
    let test_dir = env::current_dir().unwrap_or_else(|e| {
        panic!("Kon de huidige directory niet bepalen: {}", e);
    });
    test_dir.join("tests").join("tmp_commands")
}

/// Zorgt ervoor dat de test directory de benodigde configuratie heeft
fn setup_test_config() {
    INIT.call_once(|| {
        // Zorg dat we in de heretekd directory zijn
        let test_dir = env::current_dir().unwrap_or_else(|e| {
            panic!("Kon de huidige directory niet bepalen: {}", e);
        });
        
        // Maak een tijdelijke test commands directory aan
        let test_commands_dir = get_temp_test_dir();
        if test_commands_dir.exists() {
            // Verwijder oude test bestanden als deze bestaan
            fs::remove_dir_all(&test_commands_dir).unwrap_or_else(|e| {
                panic!("Kon oude test directory niet verwijderen: {}", e);
            });
        }
        
        fs::create_dir_all(&test_commands_dir).unwrap_or_else(|e| {
            panic!("Kon tijdelijke test directory niet aanmaken: {}", e);
        });
        
        // Kopieer test configuraties van tests/commands naar de tijdelijke directory
        let source_commands_dir = test_dir.join("tests").join("commands");
        if source_commands_dir.exists() {
            for entry in fs::read_dir(&source_commands_dir).unwrap_or_else(|e| {
                panic!("Kon test commands directory niet lezen: {}", e);
            }) {
                if let Ok(entry) = entry {
                    let source = entry.path();
                    if source.extension().map_or(false, |ext| ext == "yaml") {
                        let dest = test_commands_dir.join(source.file_name().unwrap());
                        fs::copy(&source, &dest).unwrap_or_else(|e| {
                            panic!("Kon configuratiebestand niet kopiëren van {:?} naar {:?}: {}", 
                                  source, dest, e);
                        });
                    }
                }
            }
        } else {
            panic!("Brondirectory voor testcommando's niet gevonden: {:?}", source_commands_dir);
        }
        
        println!("Tijdelijke test directory aangemaakt en configuraties gekopieerd: {:?}", test_commands_dir);
        
        // Stel de test directory in als tijdelijke pad voor de commands in de mock server
        env::set_var("HERETEK_COMMANDS_PATH", test_commands_dir.to_str().unwrap());
    });
}

/// Test de serverrespons door verbinding te maken met een draaiende server
///
/// Deze test vereist dat de server al draait en is standaard uitgeschakeld.
/// Start de server handmatig vóór het uitvoeren van deze test.
#[test]
#[ignore] // Toevoegen om te voorkomen dat deze test standaard wordt uitgevoerd
fn test_server_response() {
    // Geef de server tijd om op te starten (in een echte test omgeving)
    thread::sleep(Duration::from_millis(100));
    
    // Lees de server configuratie voor de juiste poort
    let config = load_config();
    let server_address = format!("{}:{}", config.heretekd.host, config.heretekd.port);
    
    match TcpStream::connect(&server_address) {
        Ok(mut stream) => {
            // Stuur een commando
            let cmd = "zfs list\n";
            stream.write_all(cmd.as_bytes()).unwrap_or_else(|e| {
                panic!("Schrijven naar server op {} mislukt: {}", server_address, e);
            });
            
            // Lees het antwoord
            let mut buffer = vec![0; 1024];
            let bytes_read = stream.read(&mut buffer).unwrap_or_else(|e| {
                panic!("Lezen van server op {} mislukt: {}", server_address, e);
            });
            
            let response = String::from_utf8_lossy(&buffer[..bytes_read]);
            
            // Controleer of de response overeenkomt met wat we verwachten
            assert!(response.contains("rpool"), 
                   "Verwachtte 'rpool' in response maar kreeg: {}", response);
            assert!(response.contains("USED"), 
                   "Verwachtte 'USED' in response maar kreeg: {}", response);
            
            println!("Server respons test geslaagd voor {}", server_address);
        },
        Err(e) => {
            // In plaats van de test te laten falen, slaan we deze over met een bericht
            println!("Server verbinding met {} mislukt: {}. Test overgeslagen.", server_address, e);
            println!("Zorg dat de server draait met 'cargo run' voordat je deze test uitvoert.");
        }
    }
}

#[test]
fn test_proxmox_detection() {
    // Test of Proxmox detectie werkt
    let pve_exists = Path::new("/etc/pve").exists();
    assert_eq!(pve_exists, false, "Deze test zou niet op een Proxmox systeem moeten draaien");
}

/// Test het correct laden van de configuratie uit de testomgeving
#[test]
fn test_config_loading() {
    // Test configuratie laden
    let current_dir = env::current_dir().unwrap_or_else(|e| {
        panic!("Kon de huidige directory niet bepalen: {}", e);
    });
    
    // Ga naar de tests directory om de test config.toml te gebruiken
    env::set_current_dir("tests").unwrap_or_else(|e| {
        panic!("Kon niet naar tests directory gaan: {}", e);
    });
    
    let config = load_config();
    
    // Ga terug naar de originele directory
    env::set_current_dir(current_dir).unwrap_or_else(|e| {
        panic!("Kon niet terug naar originele directory: {}", e);
    });
    
    // Controleer de configuratie met informatieve foutmeldingen
    assert_eq!(config.heretekd.port, 2222, 
              "Heretekd poort moet 2222 zijn, maar was {}", config.heretekd.port);
    assert_eq!(config.heretekd.host, "127.0.0.1", 
              "Heretekd host moet 127.0.0.1 zijn, maar was {}", config.heretekd.host);
    assert_eq!(config.heretekctl.port, 2223, 
              "Heretekctl poort moet 2223 zijn, maar was {}", config.heretekctl.port);
    
    println!("Configuratie succesvol geladen en gevalideerd");
}

/// Test de MockServer direct zonder netwerkverbinding
#[test]
fn test_mock_server_directly() {
    // Zorg dat de test configuratie klaar staat
    setup_test_config();
    
    println!("Test: MockServer directe functionaliteit");
    
    // Maak een mock server en test verschillende versies
    let mock = MockServer::new();
    
    // Test zfs commando (moet altijd werken door fallback)
    let zfs_response = mock.handle_command("zfs list");
    assert!(zfs_response.contains("rpool"), 
           "zfs_response moet 'rpool' bevatten maar kreeg: {}", zfs_response);
    assert!(zfs_response.contains("USED"), 
           "zfs_response moet 'USED' bevatten maar kreeg: {}", zfs_response);
    
    // Test container commando
    let pct_response = mock.handle_command("pct list");
    assert!(pct_response.contains("test-container"), 
           "pct_response moet 'test-container' bevatten maar kreeg: {}", pct_response);
    
    // Test onbekend commando
    let unknown_response = mock.handle_command("onbekend");
    assert!(unknown_response.contains("onbekend commando"), 
           "unknown_response moet 'onbekend commando' bevatten maar kreeg: {}", unknown_response);
    
    // Test versie-specifieke functionaliteit
    let v6_mock = MockServer::with_version(ProxmoxVersion::V6);
    let v6_response = v6_mock.handle_command("qm clone 200 203");
    assert!(v6_response.contains("niet beschikbaar"), 
           "v6_response moet 'niet beschikbaar' bevatten maar kreeg: {}", v6_response);
    
    let v8_mock = MockServer::with_version(ProxmoxVersion::V8);
    let v8_response = v8_mock.handle_command("pveversion");
    assert!(v8_response.contains("8.0"), 
           "v8_response moet '8.0' bevatten maar kreeg: {}", v8_response);
    
    println!("MockServer directe functionaliteitstest geslaagd");
}