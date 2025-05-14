/// Proxmox simulatie server
/// 
/// Deze module biedt een server die Proxmox commando's simuleert
/// en geschikt is voor training en demodoeleinden.

use std::collections::HashMap;
use serde::Deserialize;

/// Versies van Proxmox VE
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProxmoxVersion {
    /// Proxmox VE 6.x
    V6,
    /// Proxmox VE 7.x
    V7,
    /// Proxmox VE 8.x
    V8,
}

impl ProxmoxVersion {
    /// Geeft het numerieke versienummer
    pub fn as_number(&self) -> u8 {
        match self {
            ProxmoxVersion::V6 => 6,
            ProxmoxVersion::V7 => 7,
            ProxmoxVersion::V8 => 8,
        }
    }
    
    /// Controleert of een specifiek commando beschikbaar is in deze versie
    pub fn supports_command(&self, command: &str, config: &CommandConfig) -> bool {
        // In Proxmox 8 ondersteunen we alle commando's
        if matches!(self, ProxmoxVersion::V8) {
            return true;
        }
        
        // Dynamische commando's ondersteunen (met generieke patroonmatching)
        if command.starts_with("pct start ") || command.starts_with("pct stop ") ||
           command.starts_with("qm start ") || command.starts_with("qm stop ") {
            return true;
        }
        
        get_command_response(config, self.as_number(), command).is_some()
    }
}

/// Type voor de verzameling van commando's en hun responses
pub type CommandMap = HashMap<String, String>;

/// Houdt de commando configuraties voor verschillende Proxmox versies bij
#[derive(Debug, Clone, Deserialize)]
pub struct CommandConfig {
    /// Commando's die gemeenschappelijk zijn voor alle versies
    #[serde(default)]
    pub common: CommandMap,
    /// Versie-specifieke commando's
    #[serde(default)]
    pub versions: HashMap<u8, CommandMap>,
}

/// Proxmox simulatie server
pub struct ProxmoxServer {
    /// De versie van Proxmox die wordt gesimuleerd
    version: ProxmoxVersion,
    /// Commando configuratie
    command_config: CommandConfig,
}

impl ProxmoxServer {
    /// Maakt een nieuwe ProxmoxServer met standaardversie (Proxmox VE 8.x)
    pub fn new() -> Self {
        Self {
            version: ProxmoxVersion::V8,
            command_config: Self::load_command_config(),
        }
    }
    
    /// Maak een nieuwe ProxmoxServer met een specifieke Proxmox-versie
    pub fn with_version(version: ProxmoxVersion) -> Self {
        Self {
            version,
            command_config: Self::load_command_config(),
        }
    }
    
    /// Laadt de commando configuratie uit de YAML-bestanden
    fn load_command_config() -> CommandConfig {
        // Controleer of er een aangepast pad is opgegeven via een omgevingsvariabele
        let mut possible_paths = Vec::new();
        
        if let Ok(custom_path) = std::env::var("HERETEK_COMMANDS_PATH") {
            possible_paths.push(custom_path);
        }
        
        // Voeg standaard paden toe
        possible_paths.extend_from_slice(&[
            "commands".to_string(),
            "heretekd/commands".to_string(), 
            "../commands".to_string()
        ]);
        
        for path in possible_paths.iter() {
            let loader = CommandLoader::new(path);
            match loader.load_all_commands() {
                Ok(config) => return config,
                Err(err) => {
                    eprintln!("Kon commando's niet laden uit {}: {}", path, err);
                    // Probeer de volgende pad
                }
            }
        }
        
        // Fallback naar een lege configuratie als geen enkele pad werkt
        eprintln!("Fout bij laden commando configuratie van alle paden. Gebruik lege configuratie.");
        CommandConfig {
            common: HashMap::new(),
            versions: HashMap::new(),
        }
    }
    
    /// Geeft de huidige gesimuleerde Proxmox-versie
    pub fn get_version(&self) -> ProxmoxVersion {
        self.version
    }
    
    /// Wijzigt de gesimuleerde Proxmox-versie
    pub fn set_version(&mut self, version: ProxmoxVersion) {
        self.version = version;
    }

    /// Verwerkt een Proxmox commando en geeft het gesimuleerde antwoord terug
    pub fn handle_command(&self, input: &str) -> String {
        let trimmed = input.trim_end().trim_start_matches(|c| c == '\r' || c == '\n');
        
        // Check Proxmox versie info command
        if trimmed == "pveversion" || trimmed == "pveversion -v" {
            let detail = if trimmed.ends_with("-v") { "verbose" } else { "short" };
            return self.get_version_info(detail);
        }
        
        // Check of het een onbekend commando is
        if trimmed.starts_with("onbekend") || !trimmed.contains(' ') {
            return format!("mock: onbekend commando '{}'\n", trimmed);
        }
        
        // Controleer of het commando ondersteund wordt in deze Proxmox-versie
        if !self.version.supports_command(trimmed, &self.command_config) {
            return format!("ERROR: Commando '{}' is niet beschikbaar in Proxmox VE {:?}\n", 
                          trimmed, self.version);
        }

        // Expliciete ondersteuning voor specifieke commando's in Proxmox 8, als ze niet in de configuratie staan
        if matches!(self.version, ProxmoxVersion::V8) {
            match trimmed {
                "zfs list" => return "NAME              USED  AVAIL  REFER  MOUNTPOINT\nrpool             3.50G   96.5G   384K  /rpool\nrpool/data        1.20G   95.3G   1.20G  /rpool/data\nrpool/vm-images   2.30G   94.2G   2.30G  /rpool/vm-images\n".to_string(),
                "pct list" => return "VMID   NAME             STATUS     MEM(MB)    DISK(GB)  UPTIME\n100    test-container   running    2048       32.00     1d 4h 35m\n101    backup-server    stopped    1024       10.00     -\n".to_string(),
                "qm list" => return "VMID       NAME                 STATUS     MEM(MB)    BOOTDISK(GB) PID\n200        debian-vm            running    2048        32           12345\n201        ubuntu-vm            stopped    4096        64           -\n".to_string(),
                "pvecm nodes" => return "Node            Nodeid    Votes  Online  IP\nnode1 (local)       1      1    yes    192.168.1.100\nnode2               2      1    yes    192.168.1.101\n".to_string(),
                "qmrestore /var/lib/vz/dump/vzdump-qemu-200.vma.gz 205" => return "restoring virtual machine 200 to ID 205...\nrestore successful\n".to_string(),
                "pct restore 105 /var/lib/vz/dump/vzdump-lxc-100.tar.gz" => return "restoring container 100 to ID 105...\nrestore successful\n".to_string(),
                "vzdump 100" => return "starting backup of container 100...\nbackup successful: /var/lib/vz/dump/vzdump-lxc-100.tar.gz\n".to_string(),
                "pveam download local ubuntu-20.04-standard_20.04-1_amd64.tar.gz" => return "downloading template ubuntu-20.04-standard_20.04-1_amd64.tar.gz to local storage...\ndownload successful\n".to_string(),
                _ => {}
            }
        }
        
        // Zoek het antwoord in de configuratie
        if let Some(response) = get_command_response(&self.command_config, self.version.as_number(), trimmed) {
            return response;
        }
        
        // Dynamische commando's afhandelen voor niet-hardcoded commando's
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        
        if parts.len() > 0 {
            let command = parts[0];
            
            if parts.len() > 2 && (command == "pct" || command == "qm") {
                let action = parts[1];
                let id = parts[2];
                
                match (command, action) {
                    ("pct", "start") => {
                        return format!("starting container {}...\nstarted successfully\n", id);
                    },
                    ("pct", "stop") => {
                        return format!("stopping container {}...\nstopped successfully\n", id);
                    },
                    ("pct", "create") => {
                        return format!("creating container {}...\ncreated successfully\n", id);
                    },
                    ("pct", "destroy") => {
                        return format!("destroying container {}...\ndestroyed successfully\n", id);
                    },
                    ("pct", "migrate") => {
                        if parts.len() > 3 {
                            let target = parts[3];
                            return format!("migrating container {} to {}...\nmigration successful\n", id, target);
                        }
                    },
                    ("qm", "start") => {
                        return format!("starting virtual machine {}...\nstarted successfully\n", id);
                    },
                    ("qm", "stop") => {
                        return format!("stopping virtual machine {}...\nstopped successfully\n", id);
                    },
                    ("qm", "create") => {
                        return format!("creating virtual machine {}...\ncreated successfully\n", id);
                    },
                    ("qm", "destroy") => {
                        return format!("destroying virtual machine {}...\ndestroyed successfully\n", id);
                    },
                    ("qm", "migrate") => {
                        if parts.len() > 3 {
                            let target = parts[3];
                            return format!("migrating virtual machine {} to {}...\nmigration successful\n", id, target);
                        }
                    },
                    _ => {}
                }
                
                // Aparte afhandeling voor snapshot commando's
                if action == "snapshot" && parts.len() > 3 {
                    let snapshot_name = parts[3];
                    if command == "pct" {
                        return format!("creating snapshot {} of container {}...\nsnapshot created successfully\n", snapshot_name, id);
                    } else if command == "qm" {
                        return format!("creating snapshot {} of virtual machine {}...\nsnapshot created successfully\n", snapshot_name, id);
                    }
                }
                
                // Aparte afhandeling voor rollback commando's
                if action == "rollback" && parts.len() > 3 {
                    let snapshot_name = parts[3];
                    if command == "pct" {
                        return format!("rolling back container {} to snapshot {}...\nrollback successful\n", id, snapshot_name);
                    } else if command == "qm" {
                        return format!("rolling back virtual machine {} to snapshot {}...\nrollback successful\n", id, snapshot_name);
                    }
                }
                
                // Aparte afhandeling voor clone commando's
                if action == "clone" && parts.len() > 3 {
                    let target_id = parts[3];
                    if command == "qm" {
                        return format!("cloning VM {} to {}...\ncloning successful\n", id, target_id);
                    }
                }
            }
        }
        
        // Als we hier komen, is het een onbekend commando
        format!("mock: onbekend commando '{}'\n", trimmed)
    }
    
    /// Geeft versie-informatie terug
    fn get_version_info(&self, detail: &str) -> String {
        match self.version {
            ProxmoxVersion::V6 => {
                if detail == "verbose" {
                    "proxmox-ve: 6.4-1 (running kernel: 5.4.157-1-pve)\npve-manager: 6.4-15 (running version: 6.4-15/765fa98a)\n[...]".to_string()
                } else {
                    "pve-manager/6.4-15/765fa98a (running kernel: 5.4.157-1-pve)\n".to_string()
                }
            },
            ProxmoxVersion::V7 => {
                if detail == "verbose" {
                    "proxmox-ve: 7.4-1 (running kernel: 5.15.102-1-pve)\npve-manager: 7.4-3 (running version: 7.4-3/09ced5c1)\n[...]".to_string()
                } else {
                    "pve-manager/7.4-3/09ced5c1 (running kernel: 5.15.102-1-pve)\n".to_string()
                }
            },
            ProxmoxVersion::V8 => {
                if detail == "verbose" {
                    "proxmox-ve: 8.0-2 (running kernel: 6.2.16-3-pve)\npve-manager: 8.0.4 (running version: 8.0.4/a3df58f2)\n[...]".to_string()
                } else {
                    "pve-manager/8.0.4/a3df58f2 (running kernel: 6.2.16-3-pve)\n".to_string()
                }
            }
        }
    }
}

/// Module configuratie voor het inladen van commands
pub struct CommandLoader {
    /// Basis directory waar de command definities staan
    commands_dir: std::path::PathBuf,
    /// Categorieën van commando's (container, vm, storage, etc.)
    categories: Vec<String>,
}

impl CommandLoader {
    /// Maakt een nieuwe CommandLoader instantie
    ///
    /// # Arguments
    ///
    /// * `commands_dir` - Pad naar de directory met commando definities
    pub fn new<P: AsRef<std::path::Path>>(commands_dir: P) -> Self {
        Self {
            commands_dir: commands_dir.as_ref().to_path_buf(),
            categories: vec![
                "container".to_string(),
                "storage".to_string(),
                "system".to_string(),
                "vm".to_string(),
                "version".to_string(),
            ],
        }
    }

    /// Laadt alle commando configuraties
    ///
    /// Combineert de gemeenschappelijke commando's en de versie-specifieke commando's
    pub fn load_all_commands(&self) -> Result<CommandConfig, String> {
        let mut common = CommandMap::new();
        let mut versions: HashMap<u8, CommandMap> = HashMap::new();
    
        // Ondersteunde versies
        let supported_versions = vec![6, 7, 8];
    
        // Laad eerst alle gemeenschappelijke commando's
        for category in &self.categories {
            let category_path = self.commands_dir.join("common").join(format!("{}.yaml", category));
            if category_path.exists() {
                let category_commands = self.load_yaml_commands(&category_path)
                    .map_err(|e| format!("Fout bij laden van gemeenschappelijke commando's in {}: {}", category, e))?;
                common.extend(category_commands);
            }
        }
    
        // Probeer algemeen common.yaml bestand (buiten categorieën) te laden
        let common_path = self.commands_dir.join("common.yaml");
        if common_path.exists() {
            let common_commands = self.load_yaml_commands(&common_path)
                .map_err(|e| format!("Fout bij laden van algemeen common.yaml bestand: {}", e))?;
            common.extend(common_commands);
        }
    
        // Laad vervolgens alle versie-specifieke commando's
        for version in supported_versions {
            let mut version_commands = CommandMap::new();
            let version_dir = format!("v{}", version);
        
            for category in &self.categories {
                let category_path = self.commands_dir.join(&version_dir).join(format!("{}.yaml", category));
                if category_path.exists() {
                    let category_commands = self.load_yaml_commands(&category_path)
                        .map_err(|e| format!("Fout bij laden van {} commando's voor versie {}: {}", category, version, e))?;
                    version_commands.extend(category_commands);
                }
            }
        
            if !version_commands.is_empty() {
                versions.insert(version, version_commands);
            }
        }
    
        Ok(CommandConfig { common, versions })
    }
    
    /// Laad commando's uit een YAML bestand
    ///
    /// # Arguments
    ///
    /// * `path` - Pad naar het YAML bestand met commando definities
    fn load_yaml_commands(&self, path: &std::path::Path) -> Result<CommandMap, String> {
        if !path.exists() {
            return Err(format!("Commando definitie bestand niet gevonden: {:?}", path));
        }
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Kon bestand niet lezen: {:?} - {}", path, e))?;
    
        let yaml_commands: HashMap<String, String> = serde_yaml::from_str(&content)
            .map_err(|e| format!("Kon YAML niet parsen in bestand: {:?} - {}", path, e))?;
    
        Ok(yaml_commands)
    }
}

/// Geeft response voor een commando, gebaseerd op de Proxmox versie
///
/// # Arguments
///
/// * `config` - De CommandConfig met alle commando definities
/// * `version` - De Proxmox versie (6, 7, of 8)
/// * `command` - Het commando waarvoor een response wordt gezocht
pub fn get_command_response(config: &CommandConfig, version: u8, command: &str) -> Option<String> {
    // Controleer eerst in de versie-specifieke commando's
    if let Some(version_commands) = config.versions.get(&version) {
        if let Some(response) = version_commands.get(command) {
            return Some(response.clone());
        }
    }
    
    // Als niet gevonden in versie-specifieke commando's, probeer de gemeenschappelijke commando's
    if let Some(response) = config.common.get(command) {
        return Some(response.clone());
    }
    
    // Niet gevonden
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let default_mock = ProxmoxServer::new();
        assert_eq!(default_mock.get_version(), ProxmoxVersion::V8);
        
        let v7_mock = ProxmoxServer::with_version(ProxmoxVersion::V7);
        assert_eq!(v7_mock.get_version(), ProxmoxVersion::V7);
        
        let mut mock = ProxmoxServer::new();
        mock.set_version(ProxmoxVersion::V6);
        assert_eq!(mock.get_version(), ProxmoxVersion::V6);
    }
    
    #[test]
    fn test_version_info_command() {
        let v6_mock = ProxmoxServer::with_version(ProxmoxVersion::V6);
        let v7_mock = ProxmoxServer::with_version(ProxmoxVersion::V7);
        let v8_mock = ProxmoxServer::with_version(ProxmoxVersion::V8);
        
        let v6_info = v6_mock.handle_command("pveversion");
        let v7_info = v7_mock.handle_command("pveversion");
        let v8_info = v8_mock.handle_command("pveversion");
        
        assert!(v6_info.contains("pve-manager/6.4"));
        assert!(v7_info.contains("pve-manager/7.4"));
        assert!(v8_info.contains("pve-manager/8.0"));
    }
}

// Standaard Rust resultaat type
use std::result::Result;