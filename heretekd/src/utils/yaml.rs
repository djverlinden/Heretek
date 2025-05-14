use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

pub type CommandVersions = HashMap<String, String>;

#[derive(Debug, Deserialize)]
pub struct CommandConfig {
    #[serde(default)]
    #[serde(alias = "common")]
    pub common_commands: CommandVersions,
    #[serde(deserialize_with = "deserialize_version_commands")]
    #[serde(alias = "versions")]
    pub version_commands: HashMap<u8, CommandVersions>,
}

pub fn load_command_yaml(command_type: &str) -> CommandConfig {
    // Check if a custom path is specified via environment variable
    let mut yaml_paths = Vec::new();
    
    if let Ok(custom_path) = std::env::var("HERETEK_COMMANDS_PATH") {
        yaml_paths.push(format!("{}/{}.yaml", custom_path, command_type));
    }
    
    // Add default paths
    yaml_paths.extend_from_slice(&[
        format!("commands/{}.yaml", command_type),
        format!("heretekd/commands/{}.yaml", command_type),
        format!("../commands/{}.yaml", command_type),
    ]);
    
    let mut yaml_content = None;
    let mut error_message = String::new();
    
    for path in yaml_paths.iter() {
        match fs::read_to_string(path) {
            Ok(content) => {
                yaml_content = Some(content);
                break;
            }
            Err(e) => {
                error_message = format!("Kon {} commands YAML niet laden vanuit {}: {}", command_type, path, e);
            }
        }
    }
    
    let yaml_content = yaml_content.unwrap_or_else(|| panic!("{}", error_message));

    serde_yaml::from_str(&yaml_content)
        .unwrap_or_else(|e| panic!("Fout bij parsen van {} commands YAML: {}", command_type, e))
}

// Custom deserializer voor version_commands die string keys omzet naar u8
fn deserialize_version_commands<'de, D>(deserializer: D) -> Result<HashMap<u8, CommandVersions>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let string_map: HashMap<String, CommandVersions> = HashMap::deserialize(deserializer)?;
    
    let mut int_map = HashMap::new();
    for (key, value) in string_map {
        let int_key = key.parse::<u8>().map_err(|_| D::Error::custom(format!("Kan versie {} niet omzetten naar u8", key)))?;
        int_map.insert(int_key, value);
    }
    
    Ok(int_map)
}

/// Laadt alle versies uit het commando bestand
/// 
/// Deze functie laadt zowel de algemene commando's als alle versie-specifieke commando's
/// uit het opgegeven YAML bestand.
pub fn load_all_versions(command_type: &str, commands: &mut super::super::commands::ProxmoxCommands) {
    let config = load_command_yaml(command_type);
    
    // Voeg algemene commando's toe (beschikbaar in alle versies)
    for (cmd, resp) in config.common_commands {
        commands.common_commands.insert(cmd, resp);
    }
    
    // Voeg versie-specifieke commando's toe
    for (version, cmds) in config.version_commands {
        let version_map = commands.version_commands
            .entry(version)
            .or_insert_with(HashMap::new);
            
        for (cmd, resp) in cmds {
            version_map.insert(cmd, resp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ProxmoxCommands;

    #[test]
    #[ignore] // Alleen uitvoeren als de YAML bestanden beschikbaar zijn
    fn test_load_command_yaml() {
        let config = load_command_yaml("container");
        assert!(config.version_commands.contains_key(&8));
    }
    
    #[test]
    #[ignore] // Alleen uitvoeren als de YAML bestanden beschikbaar zijn
    fn test_load_all_versions() {
        let mut commands = ProxmoxCommands {
            common_commands: HashMap::new(),
            version_commands: HashMap::new(),
        };
        
        load_all_versions("container", &mut commands);
        
        // Test dat de commando's zijn geladen
        assert!(!commands.common_commands.is_empty() || !commands.version_commands.is_empty());
    }
}