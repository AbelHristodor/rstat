use crate::service::Service;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod http;
pub mod tcp;


const DEFAULT_MAX_RETRIES: u8 = 3;
const DEFAULT_TIMEOUT: u8 = 5;

/// Trait for all entities that can be healthchecked.
#[async_trait]
pub trait HealthChecker {
    async fn check(&self) -> HealthCheckResult;
}

/// HealthCheckRequest represents a request to perform a healthcheck.
#[derive(Clone)]
pub struct HealthCheckRequest {
    pub service: Service,
}

impl PartialEq for HealthCheckRequest {
    fn eq(&self, other: &Self) -> bool {
        self.service == other.service
    }
}

/// HealthCheckResult represents the result of a healthcheck.
#[derive(Debug)]
pub struct HealthCheckResult {
    pub id: Uuid,
    pub success: bool,
    pub code: u64,
    pub error: Option<String>,
}

/// Kind represents the type of healthcheck to perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Kind {
    /// An HTTP healthcheck executes a request to a specified URL.
    HTTP(http::HTTPChecker),
    /// A TCP healthcheck attempts to connect to a specified host and port.
    TCP(tcp::TCPChecker),
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
            "HTTP" => Kind::HTTP(http::HTTPChecker::default()),
            "TCP" => Kind::TCP(tcp::TCPChecker::default()),
            _ => panic!("Invalid healthcheck kind"),
        }
    }
}