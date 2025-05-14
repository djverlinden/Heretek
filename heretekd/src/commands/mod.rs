use std::collections::HashMap;

pub mod command_loader;
pub mod container;
pub mod storage;
pub mod system;
pub mod version;
pub mod vm;

#[derive(Debug, Clone)]
pub struct ProxmoxCommands {
    /// Commando's die beschikbaar zijn in alle Proxmox versies
    pub common_commands: HashMap<String, String>,
    
    /// Versie-specifieke commando's (key: versie, value: map van commando's)
    pub version_commands: HashMap<u8, HashMap<String, String>>,
}

impl ProxmoxCommands {
    /// Maakt een nieuwe ProxmoxCommands instantie met alle commando's geladen
    pub fn new() -> Self {
        let mut commands = Self {
            common_commands: HashMap::new(),
            version_commands: HashMap::new(),
        };
        
        // Voeg alle commando's toe
        container::add_commands(&mut commands);
        storage::add_commands(&mut commands);
        system::add_commands(&mut commands);
        version::add_commands(&mut commands);
        vm::add_commands(&mut commands);
        
        commands
    }
    
    /// Geeft het antwoord voor een commando voor een specifieke versie
    ///
    /// Zoekt eerst in versie-specifieke commando's en daarna in algemene commando's.
    pub fn get_response(&self, version: u8, command: &str) -> Option<String> {
        // Zoek eerst naar exacte matches
        if let Some(exact_match) = self.get_exact_match(version, command) {
            return Some(exact_match);
        }
        
        // Zoek naar commando's die beginnen met de opgegeven string
        // Dit maakt het mogelijk om bijv. "qm list" te vinden met alleen "qm"
        self.get_prefix_match(version, command)
    }
    
    /// Controleert of een commando wordt ondersteund in de opgegeven versie
    pub fn is_command_supported(&self, version: u8, command: &str) -> bool {
        self.get_response(version, command).is_some()
    }
    
    /// Krijg een lijst van alle ondersteunde versies
    pub fn get_supported_versions(&self) -> Vec<u8> {
        let mut versions: Vec<u8> = self.version_commands.keys().cloned().collect();
        versions.sort();
        versions
    }
    
    // Private helper methodes
    
    fn get_exact_match(&self, version: u8, command: &str) -> Option<String> {
        // Controleer eerst versie-specifieke commando's
        if let Some(version_map) = self.version_commands.get(&version) {
            if let Some(response) = version_map.get(command) {
                return Some(response.clone());
            }
        }
        
        // Controleer algemene commando's
        self.common_commands.get(command).cloned()
    }
    
    fn get_prefix_match(&self, version: u8, command_prefix: &str) -> Option<String> {
        // Controleer versie-specifieke commando's voor prefix match
        if let Some(version_map) = self.version_commands.get(&version) {
            for (cmd, resp) in version_map {
                if cmd.starts_with(command_prefix) {
                    return Some(resp.clone());
                }
            }
        }
        
        // Controleer algemene commando's voor prefix match
        for (cmd, resp) in &self.common_commands {
            if cmd.starts_with(command_prefix) {
                return Some(resp.clone());
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_response() {
        let mut commands = ProxmoxCommands::new();
        
        // Test algemene commando's
        commands.common_commands.insert(
            "test_command".to_string(),
            "test_response".to_string()
        );
        
        assert_eq!(
            commands.get_response(8, "test_command"),
            Some("test_response".to_string())
        );
        
        // Test versie-specifieke commando's
        let mut v8_commands = HashMap::new();
        v8_commands.insert(
            "v8_command".to_string(),
            "v8_response".to_string()
        );
        commands.version_commands.insert(8, v8_commands);
        
        assert_eq!(
            commands.get_response(8, "v8_command"),
            Some("v8_response".to_string())
        );
        
        // Test prefix matching
        commands.common_commands.insert(
            "prefix_test full".to_string(),
            "prefix_response".to_string()
        );
        
        assert_eq!(
            commands.get_response(8, "prefix_test"),
            Some("prefix_response".to_string())
        );
    }
}