use heretekd::{MockServer, ProxmoxVersion};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;

// Zorgt dat setup_test_config slechts één keer wordt uitgevoerd
static INIT: Once = Once::new();

/// Helper functie om een test configuratie directory aan te maken
///
/// Deze functie zorgt ervoor dat de testdirectory wordt aangemaakt en dat de
/// nodige YAML-configuratiebestanden beschikbaar zijn voor de tests.
fn setup_test_config() {
    INIT.call_once(|| {
        // Maak de commands directory aan
        let commands_dir = get_test_commands_dir();
        if !commands_dir.exists() {
            fs::create_dir_all(&commands_dir).unwrap_or_else(|e| {
                panic!("Kon commands directory niet aanmaken: {}", e);
            });
        }
        
        // Configuratiebestand aanmaken
        let container_yaml = commands_dir.join("container.yaml");
        if !container_yaml.exists() {
            fs::write(&container_yaml, r#"
common:
  "pct list": |
    VMID   NAME             STATUS     MEM(MB)    DISK(GB)  UPTIME
    100    test-container   running    2048       32.00     1d 4h 35m
    101    backup-server    stopped    1024       10.00     -

versions:
  7:
    "pct rollback 100 snap1": |
      rollback complete
"#).unwrap_or_else(|e| {
                panic!("Kon test container configuratie niet aanmaken: {}", e);
            });
            println!("✓ Test container.yaml bestand aangemaakt: {:?}", container_yaml);
        }
    });
}

/// Verkrijg het pad naar de test commands directory
fn get_test_commands_dir() -> PathBuf {
    Path::new("tests").join("commands")
}

/// Test de basisfunctionaliteit van de MockServer met verschillende commando's
///
/// Deze test controleert of verschillende soorten commando's correct worden verwerkt
/// door de MockServer, waaronder lijstcommando's, onbekende commando's en versie-info.
#[test]
fn test_mock_server_functionality() {
    // Zorg dat de test configuratie klaar staat
    setup_test_config();
    
    // Maak een mock server en test verschillende commando's
    let mock = MockServer::new();
    
    println!("🧪 Test 1: ZFS lijst commando");
    let zfs_response = mock.handle_command("zfs list");
    println!("📥 Respons: {}", zfs_response);
    assert!(
        zfs_response.contains("rpool"), 
        "ZFS response moet 'rpool' bevatten maar kreeg: {}", zfs_response
    );
    
    println!("🧪 Test 2: Container lijst commando");
    let pct_response = mock.handle_command("pct list");
    println!("📥 Respons: {}", pct_response);
    assert!(
        pct_response.contains("test-container"), 
        "PCT response moet 'test-container' bevatten maar kreeg: {}", pct_response
    );
    
    println!("🧪 Test 3: VM lijst commando");
    let qm_response = mock.handle_command("qm list");
    println!("📥 Respons: {}", qm_response);
    assert!(
        qm_response.contains("VMID"), 
        "QM response moet 'VMID' bevatten maar kreeg: {}", qm_response
    );
    
    println!("🧪 Test 4: Onbekend commando");
    let unknown_response = mock.handle_command("onbekend");
    println!("📥 Respons: {}", unknown_response);
    assert!(
        unknown_response.contains("onbekend commando"), 
        "Unknown response moet 'onbekend commando' bevatten maar kreeg: {}", unknown_response
    );
    
    println!("🧪 Test 5: Versie commando");
    let version_response = mock.handle_command("pveversion");
    println!("📥 Respons: {}", version_response);
    assert!(
        version_response.contains("pve-manager"), 
        "Version response moet 'pve-manager' bevatten maar kreeg: {}", version_response
    );
    
    println!("✅ Alle testen geslaagd!");
}

/// Test het gedrag van verschillende Proxmox versies in de MockServer
///
/// Deze test controleert of de versie-specifieke functionaliteit correct werkt,
/// zoals het wel of niet ondersteunen van bepaalde commando's in verschillende
/// Proxmox-versies.
#[test]
fn test_different_versions() {
    // Zorg dat de test configuratie klaar staat
    setup_test_config();
    
    // Test V6 versie
    let v6_mock = MockServer::with_version(ProxmoxVersion::V6);
    
    println!("🧪 Test V6: Versie info");
    let v6_version = v6_mock.handle_command("pveversion");
    println!("📥 Respons: {}", v6_version);
    assert!(
        v6_version.contains("6.4"), 
        "V6 versie moet '6.4' bevatten maar kreeg: {}", v6_version
    );
    
    println!("🧪 Test V6: Niet-ondersteund commando");
    let v6_unsupported = v6_mock.handle_command("qm clone 200 203");
    println!("📥 Respons: {}", v6_unsupported);
    assert!(
        v6_unsupported.contains("niet beschikbaar"), 
        "V6 moet aangeven dat sommige commando's niet beschikbaar zijn, maar kreeg: {}", v6_unsupported
    );
    
    // Test V8 versie
    let v8_mock = MockServer::with_version(ProxmoxVersion::V8);
    
    println!("🧪 Test V8: Versie info");
    let v8_version = v8_mock.handle_command("pveversion");
    println!("📥 Respons: {}", v8_version);
    assert!(
        v8_version.contains("8.0"), 
        "V8 versie moet '8.0' bevatten maar kreeg: {}", v8_version
    );
    
    println!("🧪 Test V8: Ondersteund commando");
    let v8_supported = v8_mock.handle_command("qm clone 200 203");
    println!("📥 Respons: {}", v8_supported);
    assert!(
        v8_supported.contains("cloning"), 
        "V8 moet clone commando ondersteunen maar kreeg: {}", v8_supported
    );
    
    println!("✅ Versie-specifieke testen geslaagd!");
}