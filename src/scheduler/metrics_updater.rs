use std::time::Duration;
use tokio::task;
use tracing::{info, warn, error};

use crate::service::metrics_calculator::MetricsCalculator;

/// MetricsUpdater runs in the background to keep service metrics updated
pub struct MetricsUpdater {
    calculator: MetricsCalculator,
    update_interval: Duration,
}

impl MetricsUpdater {
    pub fn new(calculator: MetricsCalculator, update_interval: Duration) -> Self {
        Self {
            calculator,
            update_interval,
        }
    }

    /// Start the metrics updater as a background task
    pub fn start(self) {
        task::spawn(async move {
            self.run().await;
        });
    }

    /// Main loop for updating metrics
    async fn run(self) {
        info!("Starting metrics updater with {} second interval", self.update_interval.as_secs());
        
        loop {
            match self.update_all_metrics().await {
                Ok(_) => {
                    info!("Successfully updated all service metrics");
                }
                Err(e) => {
                    error!("Failed to update service metrics: {}", e);
                }
            }

            tokio::time::sleep(self.update_interval).await;
        }
    }

    /// Update metrics for all services
    pub async fn update_all_metrics(&self) -> Result<(), anyhow::Error> {
        self.calculator.update_all_service_metrics().await
    }

    /// Update metrics for a specific service
    pub async fn update_service_metrics(&self, service_id: uuid::Uuid) -> Result<(), anyhow::Error> {
        match self.calculator.calculate_today_metrics(service_id).await {
            Ok(metric) => {
                info!("Updated metrics for service {}: {:.2}% uptime, {}ms latency", 
                      service_id, metric.uptime_percentage, metric.average_latency_ms);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to update metrics for service {}: {}", service_id, e);
                Err(e)
            }
        }
    }

    /// Calculate metrics for yesterday (useful for daily summaries)
    pub async fn calculate_yesterday_metrics(&self) -> Result<(), anyhow::Error> {
        // Get all services
        let services = crate::service::db::all(&self.calculator.pool).await?;
        
        for service in services {
            if let Err(e) = self.calculator.calculate_yesterday_metrics(service.id).await {
                warn!("Failed to calculate yesterday metrics for service {}: {}", service.name, e);
            }
        }

        Ok(())
    }

    /// Clean up old metrics (run periodically)
    pub async fn cleanup_old_metrics(&self, older_than_days: u32) -> Result<u64, anyhow::Error> {
        let deleted_count = crate::service::metrics_db::delete_old_metrics(
            &self.calculator.pool,
            older_than_days,
        )
        .await?;

        info!("Cleaned up {} old metric records (older than {} days)", deleted_count, older_than_days);
        Ok(deleted_count)
    }
} 