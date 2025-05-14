use heretekd::{MockServer, ProxmoxVersion};
use std::fs;
use std::path::Path;

// Helper functie om een test configuratie directory aan te maken
fn setup_test_config() {
    // Maak de commands directory aan
    let commands_dir = Path::new("tests/commands");
    if !commands_dir.exists() {
        fs::create_dir_all(commands_dir).expect("Kon commands directory niet aanmaken");
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
"#).expect("Kon test container configuratie niet aanmaken");
        println!("Test container.yaml bestand aangemaakt: {:?}", container_yaml);
    }
}

#[test]
fn test_mock_server_functionality() {
    // Zorg dat de test configuratie klaar staat
    setup_test_config();
    
    // Maak een mock server en test verschillende commando's
    let mock = MockServer::new();
    
    println!("Test 1: ZFS lijst commando");
    let zfs_response = mock.handle_command("zfs list");
    println!("Respons: {}", zfs_response);
    assert!(zfs_response.contains("rpool"), "ZFS response moet 'rpool' bevatten");
    
    println!("Test 2: Container lijst commando");
    let pct_response = mock.handle_command("pct list");
    println!("Respons: {}", pct_response);
    assert!(pct_response.contains("test-container"), "PCT response moet 'test-container' bevatten");
    
    println!("Test 3: VM lijst commando");
    let qm_response = mock.handle_command("qm list");
    println!("Respons: {}", qm_response);
    assert!(qm_response.contains("VMID"), "QM response moet 'VMID' bevatten");
    
    println!("Test 4: Onbekend commando");
    let unknown_response = mock.handle_command("onbekend");
    println!("Respons: {}", unknown_response);
    assert!(unknown_response.contains("onbekend commando"), "Unknown response moet 'onbekend commando' bevatten");
    
    println!("Test 5: Versie commando");
    let version_response = mock.handle_command("pveversion");
    println!("Respons: {}", version_response);
    assert!(version_response.contains("pve-manager"), "Version response moet 'pve-manager' bevatten");
    
    println!("Alle testen geslaagd!");
}

#[test]
fn test_different_versions() {
    // Test V6 versie
    let v6_mock = MockServer::with_version(ProxmoxVersion::V6);
    
    println!("Test V6: Versie info");
    let v6_version = v6_mock.handle_command("pveversion");
    println!("Respons: {}", v6_version);
    assert!(v6_version.contains("6.4"), "V6 versie moet '6.4' bevatten");
    
    println!("Test V6: Niet-ondersteund commando");
    let v6_unsupported = v6_mock.handle_command("qm clone 200 203");
    println!("Respons: {}", v6_unsupported);
    assert!(v6_unsupported.contains("niet beschikbaar"), "V6 moet sommige commando's niet ondersteunen");
    
    // Test V8 versie
    let v8_mock = MockServer::with_version(ProxmoxVersion::V8);
    
    println!("Test V8: Versie info");
    let v8_version = v8_mock.handle_command("pveversion");
    println!("Respons: {}", v8_version);
    assert!(v8_version.contains("8.0"), "V8 versie moet '8.0' bevatten");
    
    println!("Test V8: Ondersteund commando");
    let v8_supported = v8_mock.handle_command("qm clone 200 203");
    println!("Respons: {}", v8_supported);
    assert!(v8_supported.contains("cloning"), "V8 moet clone commando ondersteunen");
    
    println!("Versie-specifieke testen geslaagd!");
}