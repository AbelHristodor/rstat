use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{HealthCheckResult, HealthChecker};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TCPChecker {
    pub host: String,
    pub port: u16,
}

#[async_trait]
impl HealthChecker for TCPChecker {
    /// Checks the health of the TCP service
    async fn check(&self) -> Result<HealthCheckResult, anyhow::Error> {
        // let stream = TcpStream::connect((self.host.as_str(), self.port)).await;

        Ok(HealthCheckResult {
            id: uuid::Uuid::new_v4(),
            success: false,
            response_time: 0,
            code: 0,
            message: "Hello World".to_string(),
            created_at: chrono::Utc::now(),
        })
    }
}
