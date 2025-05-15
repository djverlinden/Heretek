// Exporteer de modules
pub mod container;
pub mod service;
pub mod cli;
pub mod tracker;

// Herexporteer de belangrijkste structuren voor tests
// DockerTest is eigenlijk een alias voor Container, aangezien deze functionaliteit 
// in de container module zit
pub use container::Container as DockerTest;
pub use service::DockerService;

// Exporteer ProxmoxTemplate voor gebruik in de service
#[derive(Debug, Clone)]
pub enum ProxmoxTemplate {
    Alpine,
    Debian,
    Custom(String),
}

impl ProxmoxTemplate {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Alpine => "alpine",
            Self::Debian => "debian",
            Self::Custom(name) => name,
        }
    }

    pub fn get_image_name(&self, version: &str) -> String {
        match self {
            Self::Alpine => format!("heretek-alpine-{}", version),
            Self::Debian => format!("heretek-debian-{}", version),
            Self::Custom(name) => format!("heretek-{}-{}", name, version),
        }
    }
}

// Voeg een functie toe om te controleren of Docker beschikbaar is
pub fn is_docker_available() -> bool {
    use std::process::Command;
    
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}