use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub type CommandVersions = HashMap<String, String>;

#[derive(Debug, Deserialize)]
pub struct CommandConfig {
    #[serde(deserialize_with = "deserialize_version_commands")]
    pub version_commands: HashMap<u8, CommandVersions>,
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

pub fn load_command_yaml(command_type: &str) -> CommandConfig {
    // Probeer verschillende paden voor de commands
    let paths = [
        Path::new("commands").join(format!("{}.yaml", command_type)),
        Path::new("heretekd/commands").join(format!("{}.yaml", command_type)),
        Path::new("../commands").join(format!("{}.yaml", command_type)),
    ];
    
    let mut yaml_content = None;
    let mut error_message = String::new();
    
    for path in paths.iter() {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                yaml_content = Some(content);
                break;
            }
            Err(e) => {
                error_message = format!("Kon {} commands YAML niet laden vanuit {:?}: {}", command_type, path, e);
            }
        }
    }
    
    let yaml_content = yaml_content.unwrap_or_else(|| panic!("{}", error_message));

    serde_yaml::from_str(&yaml_content)
        .unwrap_or_else(|e| panic!("Fout bij parsen van {} commands YAML: {}", command_type, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_command_yaml() {
        let config = load_command_yaml("container");
        assert!(config.version_commands.contains_key(&8));
    }
}