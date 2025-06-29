use std::fmt::Display;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::Service;

const DEFAULT_MAX_RETRIES: u8 = 3;
const DEFAULT_TIMEOUT: u8 = 5;

/// Trait for all entities that can be healthchecked.
#[async_trait]
pub trait HealthChecker {
    async fn check(&self) -> Result<HealthCheckResult, anyhow::Error>;
}

/// HealthCheckRequest represents a request to perform a healthcheck.
#[derive(Clone, Debug)]
pub struct HealthCheckRequest {
    pub service: Service,
}

impl PartialEq for HealthCheckRequest {
    fn eq(&self, other: &Self) -> bool {
        self.service == other.service
    }
}

impl HealthCheckRequest {
    pub fn new(service: Service) -> Self {
        HealthCheckRequest {
            service: service
        }
    }
}

/// HealthCheckResult represents the result of a healthcheck.
#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    pub id: Uuid,
    pub success: bool,
    pub code: u64,
    pub response_time: u128,
    pub message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Kind represents the type of healthcheck to perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Kind {
    /// An HTTP healthcheck executes a request to a specified URL.
    HTTP(HttpChecker),
    /// A TCP healthcheck attempts to connect to a specified host and port.
    TCP(TcpChecker),
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::HTTP(_) => write!(f, "HTTP"),
            Kind::TCP(_) => write!(f, "TCP"),
        }
    }
}

impl Into<String> for Kind {
    fn into(self) -> String {
        match self {
            Kind::HTTP(_) => "HTTP".to_string(),
            Kind::TCP(_) => "TCP".to_string(),
        }
    }
}

impl From<String> for Kind {
    fn from(kind: String) -> Self {
        match kind.as_str() {
            "HTTP" => Kind::HTTP(HttpChecker::default()),
            "TCP" => Kind::TCP(TcpChecker::default()),
            _ => panic!("Invalid healthcheck kind"),
        }
    }
}

/// HTTP checker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpChecker {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
    pub timeout: u8,
    pub max_retries: u8,
}

impl Default for HttpChecker {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: "GET".to_string(),
            headers: std::collections::HashMap::new(),
            body: None,
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }
}

/// TCP checker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpChecker {
    pub host: String,
    pub port: u16,
    pub timeout: u8,
    pub max_retries: u8,
}

impl Default for TcpChecker {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 80,
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }
} 