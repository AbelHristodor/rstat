use std::fs;
use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, warn, error};
use uuid::Uuid;

use rstat_core::{Kind, HttpChecker, TcpChecker};

pub mod loader;

/// YAML configuration structure for services
#[derive(Debug, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub name: String,
    pub kind: ServiceKind,
    pub interval: u64,
}

/// Service kind configuration for YAML
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ServiceKind {
    #[serde(rename = "http")]
    HTTP {
        url: String,
        method: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
        body: Option<String>,
        timeout: Option<u8>,
        max_retries: Option<u8>,
    },
    #[serde(rename = "tcp")]
    TCP {
        host: String,
        port: u16,
        timeout: Option<u8>,
        max_retries: Option<u8>,
    },
}

/// Configuration loader for services from YAML files
pub struct ConfigLoader {
    pool: PgPool,
}

impl ConfigLoader {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Load services from a YAML file and create them in the database
    pub async fn load_from_file(&self, file_path: &Path) -> Result<Vec<Uuid>, anyhow::Error> {
        info!("Loading services from YAML file: {}", file_path.display());
        
        let content = fs::read_to_string(file_path)?;
        let services: Vec<ServiceConfig> = serde_yaml::from_str(&content)?;
        
        info!("Found {} services in configuration file", services.len());
        
        let mut created_ids = Vec::new();
        
        for service_config in services {
            match self.create_service_from_config(service_config).await {
                Ok(id) => {
                    info!("Created service with ID: {}", id);
                    created_ids.push(id);
                }
                Err(e) => {
                    error!("Failed to create service: {}", e);
                    return Err(e);
                }
            }
        }
        
        info!("Successfully created {} services from configuration", created_ids.len());
        Ok(created_ids)
    }

    /// Create a service from configuration
    async fn create_service_from_config(&self, config: ServiceConfig) -> Result<Uuid, anyhow::Error> {
        let kind = match config.kind {
            ServiceKind::HTTP { url, method, headers, body, timeout, max_retries } => {
                Kind::HTTP(HttpChecker {
                    url,
                    method: method.unwrap_or_else(|| "GET".to_string()),
                    headers: headers.unwrap_or_default(),
                    body,
                    timeout: timeout.unwrap_or(5),
                    max_retries: max_retries.unwrap_or(3),
                })
            }
            ServiceKind::TCP { host, port, timeout, max_retries } => {
                Kind::TCP(TcpChecker {
                    host,
                    port,
                    timeout: timeout.unwrap_or(5),
                    max_retries: max_retries.unwrap_or(3),
                })
            }
        };

        let interval = Duration::from_secs(config.interval);
        
        rstat_service::create(&self.pool, &config.name, kind, interval).await
    }

    /// Load services from a directory containing YAML files
    pub async fn load_from_directory(&self, dir_path: &Path) -> Result<Vec<Uuid>, anyhow::Error> {
        info!("Loading services from directory: {}", dir_path.display());
        
        if !dir_path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {}", dir_path.display()));
        }
        
        let mut all_created_ids = Vec::new();
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                match self.load_from_file(&path).await {
                    Ok(ids) => all_created_ids.extend(ids),
                    Err(e) => {
                        warn!("Failed to load services from {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        info!("Successfully created {} services from directory", all_created_ids.len());
        Ok(all_created_ids)
    }

    /// Check if a service with the given name already exists
    pub async fn service_exists(&self, name: &str) -> Result<bool, anyhow::Error> {
        let services = rstat_service::all(&self.pool).await?;
        Ok(services.iter().any(|s| s.name == name))
    }

    /// Load services from YAML with duplicate checking
    pub async fn load_from_file_with_check(&self, file_path: &Path) -> Result<Vec<Uuid>, anyhow::Error> {
        info!("Loading services from YAML file with duplicate checking: {}", file_path.display());
        
        let content = fs::read_to_string(file_path)?;
        let services: Vec<ServiceConfig> = serde_yaml::from_str(&content)?;
        
        info!("Found {} services in configuration file", services.len());
        
        let mut created_ids = Vec::new();
        
        for service_config in services {
            let service_name = service_config.name.clone();
            if self.service_exists(&service_name).await? {
                warn!("Service '{}' already exists, skipping", service_name);
                continue;
            }
            
            match self.create_service_from_config(service_config).await {
                Ok(id) => {
                    info!("Created service '{}' with ID: {}", service_name, id);
                    created_ids.push(id);
                }
                Err(e) => {
                    error!("Failed to create service '{}': {}", service_name, e);
                    return Err(e);
                }
            }
        }
        
        info!("Successfully created {} new services from configuration", created_ids.len());
        Ok(created_ids)
    }
} 