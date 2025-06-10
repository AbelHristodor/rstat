use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{HealthCheckResult, HealthChecker};

pub fn new(host: String, port: u16) -> TCPChecker {
    TCPChecker { host, port }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TCPChecker {
    pub host: String,
    pub port: u16,
}

#[async_trait]
impl HealthChecker for TCPChecker {
    /// Checks the health of the TCP service
    async fn check(&self) -> HealthCheckResult {
        // let stream = TcpStream::connect((self.host.as_str(), self.port)).await;

        HealthCheckResult {
            id: uuid::Uuid::new_v4(),
            success: false,
            code: 0,
            error: Some("Hello World".to_string()),
        }
    }
}
