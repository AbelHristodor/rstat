use crate::utils;

use super::{DEFAULT_MAX_RETRIES, DEFAULT_TIMEOUT, HealthCheckResult, HealthChecker};
use async_trait::async_trait;
use http::{HeaderMap, Method};
use serde::{Deserialize, Serialize};
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

    pub fn url(mut self, url: String) -> Self {
        self.url = url;
        self
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

    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }
}

/// HTTPChecker is a health checker for HTTP services.
#[async_trait]
impl HealthChecker for HTTPChecker {
    /// Checks the health of the HTTP service
    async fn check(&self) -> HealthCheckResult {
        let client = self.client.clone();

        let max_retries = self.retries.clone().unwrap_or(DEFAULT_MAX_RETRIES);
        let mut attempts: u32 = 0;
        let mut last_error: Option<String> = None;

        // retry the healthcheck until max retries is reached
        while attempts <= max_retries as u32 {
            let method = self.method.clone().unwrap();
            let headers = self.headers.clone().unwrap_or_default();
            let body = self.body.clone().unwrap_or_default();
            let timeout = self
                .timeout
                .clone()
                .unwrap_or(std::time::Duration::from_secs(DEFAULT_TIMEOUT as u64));
            let url = self.url.clone();

            // Run the HTTP request
            let result = client
                .request(method, url)
                .headers(headers)
                .body(body)
                .timeout(timeout)
                .send()
                .await;

            match result {
                Ok(r) => {
                    return HealthCheckResult {
                        id: uuid::Uuid::new_v4(),
                        success: true,
                        code: r.status().as_u16() as u64,
                        error: None,
                    };
                }
                Err(err) => {
                    attempts += 1;
                    last_error = Some(err.to_string());

                    if attempts <= max_retries as u32 {
                        info!("Retrying... attempt {}/{}", attempts, max_retries);
                    } else {
                        info!("Max retries reached. Aborting...");
                        break;
                    }
                }
            }
        }

        HealthCheckResult {
            id: uuid::Uuid::new_v4(), // or use the service id
            success: false,
            code: 0,
            error: last_error,
        }
    }
}
