use super::ProxmoxCommands;
use crate::utils::yaml;

/// Voegt alle opslag-gerelateerde commando's toe
pub fn add_commands(commands: &mut ProxmoxCommands) {
    // Controleer of commando's al geladen zijn
    if commands.get_response(8, "pvesm status").is_some() {
        return;
    }

    // Laad alle versies uit het YAML bestand
    yaml::load_all_versions("storage", commands);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_storage_commands() {
        let mut commands = ProxmoxCommands {
            common_commands: HashMap::new(),
            version_commands: HashMap::new(),
        };
        
        add_commands(&mut commands);
        
        // Test dat commando's beschikbaar zijn in verschillende versies
        if !commands.version_commands.is_empty() {
            let version = *commands.version_commands.keys().next().unwrap();
            assert!(commands.get_response(version, "pvesm status").is_some());
        }
    }
}