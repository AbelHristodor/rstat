pub mod http;
pub mod tcp;
pub mod db;

use rstat_core::{HealthChecker, HealthCheckResult, HealthCheckRequest, Kind};
use anyhow::Result;

pub async fn perform_check(request: HealthCheckRequest) -> Result<HealthCheckResult> {
    match &request.service.kind {
        Kind::HTTP(http_checker) => {
            let checker = http::HttpHealthChecker::new(http_checker.clone());
            checker.check().await
        }
        Kind::TCP(tcp_checker) => {
            let checker = tcp::TcpHealthChecker::new(tcp_checker.clone());
            checker.check().await
        }
    }
} 