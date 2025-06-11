use crate::utils;

use super::{DEFAULT_MAX_RETRIES, DEFAULT_TIMEOUT, HealthCheckResult, HealthChecker};
use async_trait::async_trait;
use http::{HeaderMap, Method};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use tracing::info;

/// Constructor method to create a new instance
pub fn new(url: String, client: Option<reqwest::Client>) -> Result<HTTPChecker, anyhow::Error> {
    HTTPChecker {
        url,
        headers: None,
        // Defaults to GET
        method: Some(Method::default()),
        body: None,
        retries: None,
        timeout: None,
        client: client.unwrap_or_default(),
    }
    .validate()
}

/// HTTPChecker is a health checker that uses HTTP requests to check the health of a service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HTTPChecker {
    pub url: String,
    #[serde(with = "http_serde::option::method")]
    pub method: Option<Method>,
    #[serde(with = "http_serde::option::header_map")]
    pub headers: Option<HeaderMap>,
    pub body: Option<String>,
    pub retries: Option<u8>,
    pub timeout: Option<std::time::Duration>,

    #[serde(skip)]
    pub client: reqwest::Client,
}

impl HTTPChecker {
    fn validate(self) -> Result<Self, anyhow::Error> {
        utils::validate_url(&self.url)?;

        Ok(self)
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn retries(mut self, retries: u8) -> Self {
        self.retries = Some(retries);
        self
    }

    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = Some(std::time::Duration::from_secs(timeout as u64));
        self
    }
}

/// HTTPChecker is a health checker for HTTP services.
#[async_trait]
impl HealthChecker for HTTPChecker {
    /// Checks the health of the HTTP service
    async fn check(&self) -> Result<HealthCheckResult, anyhow::Error> {
        let client = self.client.clone();
        let max_retries = self.retries.unwrap_or(DEFAULT_MAX_RETRIES);
        let mut attempts: u8 = 0;
        let mut last_error: Option<String> = None;

        // Helper function to build the request
        let build_request = || {
            let method = self.method.clone().unwrap();
            let headers = self.headers.clone().unwrap_or_default();
            let body = self.body.clone().unwrap_or_default();
            let timeout = self
                .timeout
                .unwrap_or_else(|| std::time::Duration::from_secs(DEFAULT_TIMEOUT as u64));
            let url = self.url.clone();

            client
                .request(method, url)
                .headers(headers)
                .body(body)
                .timeout(timeout)
                .build()
        };

        // Retry the healthcheck until max retries is reached
        while attempts <= max_retries {
            let req = match build_request() {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(e.to_string());
                    break;
                }
            };

            let start_time = Instant::now();
            let result = client.execute(req).await;
            let elapsed = start_time.elapsed();

            match result {
                Ok(r) => {
                    let success = r.status().is_success();
                    return Ok(HealthCheckResult {
                        id: uuid::Uuid::new_v4(),
                        success,
                        response_time: elapsed.as_millis(),
                        code: r.status().as_u16() as u64,
                        message: r.text().await.unwrap_or_default(),
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
        })
    }
}
