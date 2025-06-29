use rstat_core::{HealthChecker, HealthCheckResult, HttpChecker};
use async_trait::async_trait;
use http::{HeaderMap, Method, HeaderName};
use tokio::time::Instant;
use tracing::info;

/// HTTP health checker implementation
pub struct HttpHealthChecker {
    config: HttpChecker,
    client: reqwest::Client,
}

impl HttpHealthChecker {
    pub fn new(config: HttpChecker) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    fn validate_url(url: &str) -> Result<(), anyhow::Error> {
        url::Url::parse(url)?;
        Ok(())
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (key, value) in &self.config.headers {
            if let Ok(header_name) = key.parse::<HeaderName>() {
                if let Ok(header_value) = value.parse() {
                    headers.insert(header_name, header_value);
                }
            }
        }
        headers
    }
}

#[async_trait]
impl HealthChecker for HttpHealthChecker {
    async fn check(&self) -> Result<HealthCheckResult, anyhow::Error> {
        Self::validate_url(&self.config.url)?;

        let max_retries = self.config.max_retries;
        let mut attempts: u8 = 0;
        let mut last_error: Option<String> = None;

        while attempts <= max_retries {
            let method = Method::from_bytes(self.config.method.as_bytes())?;
            let headers = self.build_headers();
            let body = self.config.body.clone().unwrap_or_default();
            let timeout = std::time::Duration::from_secs(self.config.timeout as u64);

            let req = self.client
                .request(method, &self.config.url)
                .headers(headers)
                .body(body)
                .timeout(timeout)
                .build()?;
            
            let start_time = Instant::now();
            let result = self.client.execute(req).await;
            let elapsed = start_time.elapsed();

            match result {
                Ok(r) => {
                    let success = r.status().is_success();
                    return Ok(HealthCheckResult {
                        id: uuid::Uuid::new_v4(),
                        success,
                        response_time: elapsed.as_micros(),
                        code: r.status().as_u16() as u64,
                        message: r.text().await.unwrap_or_default(),
                        created_at: chrono::Utc::now(),
                    });
                }
                Err(err) => {
                    attempts += 1;
                    last_error = Some(err.to_string());

                    if attempts <= max_retries {
                        info!("Retrying... attempt {}/{}", attempts, max_retries);
                    } else {
                        info!("Max retries reached. Aborting...");
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
            message: last_error.unwrap_or_default(),
            created_at: chrono::Utc::now(),
        })
    }
} 