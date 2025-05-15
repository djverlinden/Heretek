use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Container executor for testing and running scripts")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a specific container template
    Install {
        /// Template type (alpine or debian)
        #[arg(value_enum)]
        template: TemplateType,
        
        /// Version of the template (e.g. 3.12, 11)
        #[arg(default_value = "latest")]
        version: String,
    },
    
    /// Run a bash script in a container
    Run {
        /// Path to the bash script to execute
        script: PathBuf,
        
        /// Template to use (alpine or debian)
        #[arg(value_enum)]
        template: Option<TemplateType>,
        
        /// Version of the template (e.g. 3.12, 11)
        version: Option<String>,
        
        /// Optional timeout in seconds (default: 30)
        #[arg(short, long, default_value_t = 30)]
        timeout: u64,
        
        /// Mount a directory into the container
        #[arg(short, long)]
        mount: Option<PathBuf>,
    },

    /// Cleanup Docker containers and images
    Cleanup {
        /// Remove all heretek images, not just dangling ones
        #[arg(short, long)]
        all: bool,
        
        /// Also remove base images (alpine, debian)
        #[arg(long)]
        include_base: bool,
        
        /// Force removal of running containers
        #[arg(short, long)]
        force: bool,
    },
    
    /// List tracked Docker resources (containers and images)
    List {
        /// Show only containers
        #[arg(short, long)]
        containers: bool,
        
        /// Show only images
        #[arg(short, long)]
        images: bool,
        
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Start a long-running container
    Start {
        /// Template to use (alpine or debian)
        #[arg(value_enum)]
        template: Option<TemplateType>,
        
        /// Version of the template (e.g. 3.12, 11)
        version: Option<String>,
        
        /// Mount a directory into the container
        #[arg(short, long)]
        mount: Option<PathBuf>,
        
        /// Expose ports (format: host:container, e.g. 8080:80)
        #[arg(short, long)]
        port: Option<String>,
        
        /// Custom name for the container
        #[arg(short, long)]
        name: Option<String>,
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum TemplateType {
    Alpine,
    Debian,
}

impl TemplateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateType::Alpine => "alpine",
            TemplateType::Debian => "debian",
        }
    }
    
    pub fn get_image_name(&self, version: &str) -> String {
        if version == "latest" {
            format!("heretek-{}-latest", self.as_str())
        } else {
            format!("heretek-{}-{}", self.as_str(), version)
        }
    }
    
    #[allow(dead_code)]
    pub fn get_template_path(&self, version: &str) -> PathBuf {
        PathBuf::from(format!("templates/{}-{}.tar.xz", self.as_str(), version))
    }
}