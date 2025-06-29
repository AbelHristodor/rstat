use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::healthcheck::HealthCheckResult;
use super::metrics::{ServiceMetric, ServiceMetricsSummary};
use super::metrics_db;

/// MetricsCalculator handles the computation of service metrics from health check results
pub struct MetricsCalculator {
    pub pool: PgPool,
}

impl MetricsCalculator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Calculate and store daily metrics for a service
    pub async fn calculate_daily_metrics(
        &self,
        service_id: Uuid,
        date: NaiveDate,
    ) -> Result<ServiceMetric, anyhow::Error> {
        // Get all health check results for the service on the specified date
        let results = self.get_health_check_results_for_date(service_id, date).await?;
        
        if results.is_empty() {
            // No results for this date, create a zero metric
            let metric = ServiceMetric::new(service_id, date, 0.0, 0, 0, 0);
            let _ = metrics_db::upsert_metric(
                &self.pool,
                service_id,
                date,
                metric.uptime_percentage,
                metric.average_latency_ms,
                metric.total_checks,
                metric.successful_checks,
            )
            .await?;
            return Ok(metric);
        }

        // Calculate metrics from results
        let total_checks = results.len() as u32;
        let successful_checks = results.iter().filter(|r| r.success).count() as u32;
        let uptime_percentage = ServiceMetric::calculate_uptime_percentage(successful_checks, total_checks);
        
        // Calculate average latency (only from successful checks)
        let successful_results: Vec<&HealthCheckResult> = results.iter().filter(|r| r.success).collect();
        let average_latency_ms = if successful_results.is_empty() {
            0
        } else {
            let total_latency_microseconds: u128 = successful_results.iter().map(|r| r.response_time).sum();
            let average_latency_microseconds = total_latency_microseconds / successful_results.len() as u128;
            // Convert microseconds to milliseconds
            (average_latency_microseconds / 1000) as u32
        };

        // Create the metric
        let metric = ServiceMetric::new(
            service_id,
            date,
            uptime_percentage,
            average_latency_ms,
            total_checks,
            successful_checks,
        );

        // Store in database
        let _ = metrics_db::upsert_metric(
            &self.pool,
            service_id,
            date,
            metric.uptime_percentage,
            metric.average_latency_ms,
            metric.total_checks,
            metric.successful_checks,
        )
        .await?;

        Ok(metric)
    }

    /// Calculate metrics for today for a service
    pub async fn calculate_today_metrics(&self, service_id: Uuid) -> Result<ServiceMetric, anyhow::Error> {
        let today = Utc::now().date_naive();
        self.calculate_daily_metrics(service_id, today).await
    }

    /// Calculate metrics for yesterday for a service
    pub async fn calculate_yesterday_metrics(&self, service_id: Uuid) -> Result<ServiceMetric, anyhow::Error> {
        let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);
        self.calculate_daily_metrics(service_id, yesterday).await
    }

    /// Calculate metrics for a date range
    pub async fn calculate_metrics_for_range(
        &self,
        service_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ServiceMetric>, anyhow::Error> {
        let mut metrics = Vec::new();
        let mut current_date = start_date;

        while current_date <= end_date {
            let metric = self.calculate_daily_metrics(service_id, current_date).await?;
            metrics.push(metric);
            current_date = current_date + chrono::Duration::days(1);
        }

        Ok(metrics)
    }

    /// Get metrics summary for a service
    pub async fn get_metrics_summary(
        &self,
        service_id: Uuid,
        days: Option<u32>,
    ) -> Result<ServiceMetricsSummary, anyhow::Error> {
        // First, ensure we have calculated metrics for the requested period
        let days = days.unwrap_or(30);
        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days(days as i64);
        
        // Calculate any missing metrics in the range
        self.calculate_metrics_for_range(
            service_id,
            start_date,
            end_date,
        )
        .await?;

        // Get the summary
        let summary = metrics_db::get_metrics_summary(&self.pool, service_id, Some(days)).await?;
        Ok(summary)
    }

    /// Update metrics for all services
    pub async fn update_all_service_metrics(&self) -> Result<(), anyhow::Error> {
        // Get all services
        let services = super::db::all(&self.pool).await?;
        
        for service in services {
            if let Err(e) = self.calculate_today_metrics(service.id).await {
                tracing::warn!("Failed to calculate metrics for service {}: {}", service.name, e);
            }
        }

        Ok(())
    }

    /// Get health check results for a specific date
    async fn get_health_check_results_for_date(
        &self,
        service_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<HealthCheckResult>, anyhow::Error> {
        let start_datetime = date.and_hms_opt(0, 0, 0).unwrap();
        let end_datetime = date.and_hms_opt(23, 59, 59).unwrap();

        let rows = sqlx::query!(
            r#"
            SELECT id, success, code, response_time, message, created_at
            FROM healthcheck_results 
            WHERE service_id = $1 AND created_at >= $2 AND created_at <= $3
            ORDER BY created_at
            "#,
            service_id,
            start_datetime,
            end_datetime
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| HealthCheckResult {
                id: row.id,
                success: row.success,
                code: row.code.unwrap_or_default().parse::<u64>().unwrap_or_default(),
                response_time: row.response_time.unwrap_or_default().try_into().unwrap_or_default(),
                message: row.message.unwrap_or_default(),
                created_at: row.created_at.and_utc(),
            })
            .collect();

        Ok(results)
    }
} 