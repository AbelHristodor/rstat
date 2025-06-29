use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ServiceMetric represents daily metrics for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetric {
    pub id: Uuid,
    pub service_id: Uuid,
    #[serde(serialize_with = "serialize_date", deserialize_with = "deserialize_date")]
    pub date: NaiveDate,
    pub uptime_percentage: f64,
    pub average_latency_ms: u32,
    pub total_checks: u32,
    pub successful_checks: u32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

/// ServiceMetricsSummary represents computed metrics for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetricsSummary {
    pub service_id: Uuid,
    pub current_uptime: f64,
    pub current_latency_ms: u32,
    pub average_latency_ms: u32,
    pub uptime_data: Vec<UptimeDataPoint>,
}

/// UptimeDataPoint represents a single day's uptime and latency data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeDataPoint {
    pub date: String, // ISO date string (YYYY-MM-DD)
    pub uptime_percentage: f64,
    pub latency_ms: u32,
}

impl ServiceMetric {
    /// Create a new ServiceMetric instance
    pub fn new(
        service_id: Uuid,
        date: NaiveDate,
        uptime_percentage: f64,
        average_latency_ms: u32,
        total_checks: u32,
        successful_checks: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            service_id,
            date,
            uptime_percentage,
            average_latency_ms,
            total_checks,
            successful_checks,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Calculate uptime percentage from check results
    pub fn calculate_uptime_percentage(successful_checks: u32, total_checks: u32) -> f64 {
        if total_checks == 0 {
            0.0
        } else {
            (successful_checks as f64 / total_checks as f64) * 100.0
        }
    }
}

impl ServiceMetricsSummary {
    /// Create a new ServiceMetricsSummary from a list of metrics
    pub fn from_metrics(service_id: Uuid, metrics: Vec<ServiceMetric>) -> Self {
        if metrics.is_empty() {
            return Self {
                service_id,
                current_uptime: 0.0,
                current_latency_ms: 0,
                average_latency_ms: 0,
                uptime_data: vec![],
            };
        }

        // Sort metrics by date (most recent first)
        let mut sorted_metrics = metrics;
        sorted_metrics.sort_by(|a, b| b.date.cmp(&a.date));

        // Current metrics (most recent day)
        let current_metric = &sorted_metrics[0];
        let current_uptime = current_metric.uptime_percentage;
        let current_latency_ms = current_metric.average_latency_ms;

        // Calculate average latency across all days
        let total_latency: u32 = sorted_metrics.iter().map(|m| m.average_latency_ms).sum();
        let average_latency_ms = total_latency / sorted_metrics.len() as u32;

        // Convert to uptime data points
        let uptime_data = sorted_metrics
            .iter()
            .map(|metric| UptimeDataPoint {
                date: metric.date.format("%Y-%m-%d").to_string(),
                uptime_percentage: metric.uptime_percentage,
                latency_ms: metric.average_latency_ms,
            })
            .collect();

        Self {
            service_id,
            current_uptime,
            current_latency_ms,
            average_latency_ms,
            uptime_data,
        }
    }
}

fn serialize_date<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&date.format("%Y-%m-%d").to_string())
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
} 