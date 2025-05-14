use serde::Deserialize;
// Hernoem de config crate om naamconflicten te voorkomen
use config as cfg_crate;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub heretekd: HostPort,
    pub heretekctl: HostPort,
    #[serde(default)]
    pub proxmox: ProxmoxConfig,
}

#[derive(Debug, Deserialize)]
pub struct HostPort {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProxmoxConfig {
    pub version: Option<String>,
}

pub fn load_config() -> AppConfig {
    // Probeer eerst het configuratiebestand te laden
    let builder = cfg_crate::Config::builder();
    
    // Voeg eerst een standaard fallback configuratie toe
    let default_config = r#"[heretekd]
host = "127.0.0.1"
port = 2222

[heretekctl]
host = "127.0.0.1"
port = 9001

[proxmox]
# Versie van Proxmox die gesimuleerd wordt (6, 7, of 8)
version = "8"
"#;
    
    let builder = builder.add_source(cfg_crate::File::from_str(default_config, cfg_crate::FileFormat::Toml));
    
    // Probeer eerst het configuratiebestand in de huidige directory te laden
    let builder = match std::fs::metadata("config.toml") {
        Ok(_) => builder.add_source(cfg_crate::File::with_name("config.toml").format(cfg_crate::FileFormat::Toml)),
        Err(_) => {
            // Probeer dan in de project root directory te laden
            match std::fs::metadata("../config.toml") {
                Ok(_) => builder.add_source(cfg_crate::File::with_name("../config.toml").format(cfg_crate::FileFormat::Toml)),
                Err(_) => builder, // Als beide niet gevonden worden, gebruik alleen standaard config
            }
        }
    };
    
    // Bouw de configuratie
    let config = builder.build().unwrap_or_else(|e| {
        eprintln!("⚠️ Waarschuwing bij laden configuratie: {}. Gebruik standaard instellingen. Om de server te stoppen, gebruik Ctrl+C in de terminal of kill het proces met 'kill <PID>'.", e);
        
        // Schrijf standaard configuratiebestand als het niet bestaat
        if let Err(_) = std::fs::metadata("config.toml") {
            if let Err(write_err) = std::fs::write("config.toml", default_config) {
                eprintln!("⚠️ Kon standaard configuratiebestand niet aanmaken: {}", write_err);
            } else {
                println!("✅ Standaard configuratiebestand aangemaakt in de huidige map.");
                println!("📝 Je kunt deze configuratie bewerken in 'config.toml'");
            }
        }
        
        cfg_crate::Config::builder()
            .add_source(cfg_crate::File::from_str(default_config, cfg_crate::FileFormat::Toml))
            .build()
            .expect("Standaard configuratie zou altijd moeten werken")
    });

    // Deserialize naar AppConfig
    config.try_deserialize::<AppConfig>().unwrap_or_else(|e| {
        eprintln!("⚠️ Fout bij deserialize configuratie: {}. Gebruik standaard instellingen. Om de server te stoppen, gebruik Ctrl+C in de terminal of kill het proces met 'kill <PID>'.", e);
        
        // Parse standaard configuratie direct
        let default = cfg_crate::Config::builder()
            .add_source(cfg_crate::File::from_str(default_config, cfg_crate::FileFormat::Toml))
            .build()
            .expect("Standaard configuratie zou altijd moeten werken")
            .try_deserialize::<AppConfig>()
            .expect("Standaard configuratie zou altijd moeten deserializeren");
            
        default
    })
}
