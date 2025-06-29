use rstat_core::{HealthChecker, HealthCheckResult, TcpChecker};
use async_trait::async_trait;
use tokio::time::Instant;
use tokio::net::TcpStream;

/// TCP health checker implementation
pub struct TcpHealthChecker {
    config: TcpChecker,
}

impl TcpHealthChecker {
    pub fn new(config: TcpChecker) -> Self {
        Self { config }
    }
}

#[async_trait]
impl HealthChecker for TcpHealthChecker {
    async fn check(&self) -> Result<HealthCheckResult, anyhow::Error> {
        let max_retries = self.config.max_retries;
        let mut attempts: u8 = 0;
        let mut last_error: Option<String> = None;

        while attempts <= max_retries {
            let start_time = Instant::now();
            let result = TcpStream::connect((self.config.host.as_str(), self.config.port)).await;
            let elapsed = start_time.elapsed();

            match result {
                Ok(_stream) => {
                    return Ok(HealthCheckResult {
                        id: uuid::Uuid::new_v4(),
                        success: true,
                        response_time: elapsed.as_micros(),
                        code: 200,
                        message: "TCP connection successful".to_string(),
                        created_at: chrono::Utc::now(),
                    });
                }
                Err(err) => {
                    attempts += 1;
                    last_error = Some(err.to_string());

                    if attempts <= max_retries {
                        tracing::info!("Retrying TCP connection... attempt {}/{}", attempts, max_retries);
                    } else {
                        tracing::info!("Max retries reached for TCP connection. Aborting...");
                        break;
                    }
                }
            }
        }

        Ok(HealthCheckResult {
            id: uuid::Uuid::new_v4(),
            success: false,
            response_time: 0,
            code: 0,
            message: last_error.unwrap_or_else(|| "TCP connection failed".to_string()),
            created_at: chrono::Utc::now(),
        })
    }
} 