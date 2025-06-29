use std::path::Path;
use tracing::{info, warn};
use anyhow::Result;

use super::ConfigLoader;

impl ConfigLoader {
    /// Load services from a default configuration file path
    pub async fn load_from_default(&self) -> Result<Vec<uuid::Uuid>, anyhow::Error> {
        let default_paths = [
            "config/services.yaml",
            "config/services.yml", 
            "services.yaml",
            "services.yml",
        ];
        
        for path in &default_paths {
            let path = Path::new(path);
            if path.exists() {
                info!("Found default configuration file: {}", path.display());
                return self.load_from_file_with_check(path).await;
            }
        }
        
        warn!("No default configuration file found in: {:?}", default_paths);
        Ok(Vec::new())
    }

    /// Load services from a default configuration directory
    pub async fn load_from_default_directory(&self) -> Result<Vec<uuid::Uuid>, anyhow::Error> {
        let default_dirs = [
            "config/services",
            "config",
            "services",
        ];
        
        for dir in &default_dirs {
            let dir = Path::new(dir);
            if dir.exists() && dir.is_dir() {
                info!("Found default configuration directory: {}", dir.display());
                return self.load_from_directory(dir).await;
            }
        }
        
        warn!("No default configuration directory found in: {:?}", default_dirs);
        Ok(Vec::new())
    }

    /// Load services from environment-specified configuration
    pub async fn load_from_env(&self) -> Result<Vec<uuid::Uuid>, anyhow::Error> {
        if let Ok(config_path) = std::env::var("RSTAT_CONFIG_PATH") {
            let path = Path::new(&config_path);
            if path.exists() {
                info!("Loading configuration from environment path: {}", path.display());
                if path.is_file() {
                    return self.load_from_file_with_check(path).await;
                } else if path.is_dir() {
                    return self.load_from_directory(path).await;
                }
            } else {
                warn!("Environment config path does not exist: {}", path.display());
            }
        }
        
        Ok(Vec::new())
    }
} 