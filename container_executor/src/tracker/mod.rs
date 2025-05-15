use std::path::PathBuf;
use sled::{Db, IVec};
use serde::{Serialize, Deserialize};

const CONTAINER_PREFIX: &str = "container:";
const IMAGE_PREFIX: &str = "image:";

#[derive(Debug, Serialize, Deserialize)]
enum ResourceType {
    Container,
    Image,
}

#[derive(Debug, Serialize, Deserialize)]
struct Resource {
    resource_type: ResourceType,
    id: String,
    timestamp: u64,
}

/// ResourceTracker houdt bij welke Docker resources door de applicatie zijn aangemaakt
/// en biedt functionaliteit om deze op te schonen.
pub struct ResourceTracker {
    db: Db,
}

impl ResourceTracker {
    /// Maak een nieuwe ResourceTracker aan of laad een bestaande
    pub fn new() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let db_path = home_dir.join(".heretek").join("resources.db");
        
        // Zorg ervoor dat de directory bestaat
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap_or_else(|_| {
                    eprintln!("Kon directory {} niet aanmaken, resources worden niet bijgehouden", 
                            parent.display());
                });
            }
        }
        
        let db = sled::open(db_path).unwrap_or_else(|e| {
            eprintln!("Kon database niet openen: {}. Een tijdelijke database wordt gebruikt.", e);
            sled::Config::new()
                .temporary(true)
                .open()
                .expect("Kon geen tijdelijke database aanmaken")
        });
        
        Self { db }
    }
    
    /// Volg een nieuwe container
    pub fn track_container(&mut self, container_id: &str) {
        let resource = Resource {
            resource_type: ResourceType::Container,
            id: container_id.trim().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        if let Ok(serialized) = serde_json::to_string(&resource) {
            let key = format!("{}{}", CONTAINER_PREFIX, container_id.trim());
            if let Err(e) = self.db.insert(key, serialized.as_bytes()) {
                eprintln!("Kon container {} niet bijhouden: {}", container_id, e);
            }
            let _ = self.db.flush();
        }
    }
    
    /// Volg een nieuwe image
    pub fn track_image(&mut self, image_name: &str) {
        let resource = Resource {
            resource_type: ResourceType::Image,
            id: image_name.trim().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        if let Ok(serialized) = serde_json::to_string(&resource) {
            let key = format!("{}{}", IMAGE_PREFIX, image_name.trim());
            if let Err(e) = self.db.insert(key, serialized.as_bytes()) {
                eprintln!("Kon image {} niet bijhouden: {}", image_name, e);
            }
            let _ = self.db.flush();
        }
    }
    
    /// Geef lijst van containers terug
    pub fn get_containers(&self) -> Vec<String> {
        let mut containers = Vec::new();
        
        // Zoek naar sleutels met het container prefix
        let prefix = CONTAINER_PREFIX.as_bytes();
        
        // In sled is scan_prefix() directe iteratie, geen Result
        for result in self.db.scan_prefix(prefix) {
            if let Ok((_, value)) = result {
                if let Ok(value_str) = std::str::from_utf8(&value) {
                    if let Ok(resource) = serde_json::from_str::<Resource>(value_str) {
                        containers.push(resource.id);
                    }
                }
            }
        }
        
        containers
    }
    
    /// Geef lijst van images terug
    pub fn get_images(&self) -> Vec<String> {
        let mut images = Vec::new();
        
        // Zoek naar sleutels met het image prefix
        let prefix = IMAGE_PREFIX.as_bytes();
        
        // In sled is scan_prefix() directe iteratie, geen Result
        for result in self.db.scan_prefix(prefix) {
            if let Ok((_, value)) = result {
                if let Ok(value_str) = std::str::from_utf8(&value) {
                    if let Ok(resource) = serde_json::from_str::<Resource>(value_str) {
                        images.push(resource.id);
                    }
                }
            }
        }
        
        images
    }
    
    /// Verwijder container uit de track lijst
    pub fn remove_container(&mut self, container_id: &str) {
        let key = format!("{}{}", CONTAINER_PREFIX, container_id.trim());
        if let Err(e) = self.db.remove(key) {
            eprintln!("Kon container {} niet uit de lijst verwijderen: {}", container_id, e);
        }
        let _ = self.db.flush();
    }
    
    /// Verwijder image uit de track lijst
    pub fn remove_image(&mut self, image_name: &str) {
        let key = format!("{}{}", IMAGE_PREFIX, image_name.trim());
        if let Err(e) = self.db.remove(key) {
            eprintln!("Kon image {} niet uit de lijst verwijderen: {}", image_name, e);
        }
        let _ = self.db.flush();
    }
    
    /// Verwijder alle containers uit de track lijst
    #[allow(dead_code)]
    pub fn clear_containers(&mut self) {
        let prefix = CONTAINER_PREFIX.as_bytes();
        let keys_to_remove: Vec<IVec> = self.db.scan_prefix(prefix)
            .filter_map(Result::ok)
            .map(|(key, _)| key)
            .collect();
            
        for key in keys_to_remove {
            let _ = self.db.remove(key);
        }
        
        let _ = self.db.flush();
    }
    
    /// Verwijder alle images uit de track lijst
    #[allow(dead_code)]
    pub fn clear_images(&mut self) {
        let prefix = IMAGE_PREFIX.as_bytes();
        let keys_to_remove: Vec<IVec> = self.db.scan_prefix(prefix)
            .filter_map(Result::ok)
            .map(|(key, _)| key)
            .collect();
            
        for key in keys_to_remove {
            let _ = self.db.remove(key);
        }
        
        let _ = self.db.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    
    #[test]
    fn test_track_container() {
        // Gebruik een tijdelijke database voor tests
        let temp_dir = env::temp_dir().join("heretek_test");
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");
        
        // Verwijder eventuele bestaande database
        let _ = fs::remove_dir_all(&db_path);
        
        let db = sled::open(&db_path).unwrap();
        let mut tracker = ResourceTracker { db };
        
        // Test container tracking
        tracker.track_container("test-container-id");
        
        // Controleer of de container is bijgehouden
        let containers = tracker.get_containers();
        assert!(containers.contains(&"test-container-id".to_string()));
        
        // Cleanup
        fs::remove_dir_all(temp_dir).unwrap_or_default();
    }
}