use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use num_traits::{ToPrimitive, FromPrimitive};

use crate::models::{ServiceMetric, ServiceMetricsSummary};

/// Upsert a service metric for a specific date
pub async fn upsert_metric(
    pool: &PgPool,
    service_id: Uuid,
    date: NaiveDate,
    uptime_percentage: f64,
    average_latency_ms: u32,
    total_checks: u32,
    successful_checks: u32,
) -> Result<Uuid, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO service_metrics (service_id, date, uptime_percentage, average_latency_ms, total_checks, successful_checks)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (service_id, date) 
        DO UPDATE SET 
            uptime_percentage = EXCLUDED.uptime_percentage,
            average_latency_ms = EXCLUDED.average_latency_ms,
            total_checks = EXCLUDED.total_checks,
            successful_checks = EXCLUDED.successful_checks,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id
        "#,
        service_id,
        date,
        BigDecimal::from_f64(uptime_percentage).unwrap_or(BigDecimal::from(0)),
        average_latency_ms as i32,
        total_checks as i32,
        successful_checks as i32
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

/// Get metrics for a service within a date range
pub async fn get_metrics_for_service(
    pool: &PgPool,
    service_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<ServiceMetric>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, service_id, date, uptime_percentage, average_latency_ms, 
               total_checks, successful_checks, created_at, updated_at
        FROM service_metrics 
        WHERE service_id = $1 AND date >= $2 AND date <= $3
        ORDER BY date DESC
        "#,
        service_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    let metrics = rows
        .into_iter()
        .map(|row| ServiceMetric {
            id: row.id,
            service_id: row.service_id,
            date: row.date,
            uptime_percentage: row.uptime_percentage.to_f64().unwrap_or(0.0),
            average_latency_ms: row.average_latency_ms as u32,
            total_checks: row.total_checks as u32,
            successful_checks: row.successful_checks as u32,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
        })
        .collect();

    Ok(metrics)
}

/// Get metrics for the last N days for a service
pub async fn get_metrics_for_service_last_days(
    pool: &PgPool,
    service_id: Uuid,
    days: u32,
) -> Result<Vec<ServiceMetric>, sqlx::Error> {
    let end_date = Utc::now().date_naive();
    let start_date = end_date - chrono::Duration::days(days as i64);
    
    get_metrics_for_service(
        pool,
        service_id,
        start_date,
        end_date,
    )
    .await
}

/// Get metrics summary for a service (last 30 days by default)
pub async fn get_metrics_summary(
    pool: &PgPool,
    service_id: Uuid,
    days: Option<u32>,
) -> Result<ServiceMetricsSummary, sqlx::Error> {
    let days = days.unwrap_or(30);
    let metrics = get_metrics_for_service_last_days(pool, service_id, days).await?;
    
    Ok(ServiceMetricsSummary::from_metrics(service_id, metrics))
}

/// Get metrics for all services within a date range
pub async fn get_all_metrics(
    pool: &PgPool,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<ServiceMetric>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, service_id, date, uptime_percentage, average_latency_ms, 
               total_checks, successful_checks, created_at, updated_at
        FROM service_metrics 
        WHERE date >= $1 AND date <= $2
        ORDER BY service_id, date DESC
        "#,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    let metrics = rows
        .into_iter()
        .map(|row| ServiceMetric {
            id: row.id,
            service_id: row.service_id,
            date: row.date,
            uptime_percentage: row.uptime_percentage.to_f64().unwrap_or(0.0),
            average_latency_ms: row.average_latency_ms as u32,
            total_checks: row.total_checks as u32,
            successful_checks: row.successful_checks as u32,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
        })
        .collect();

    Ok(metrics)
}

/// Get metrics for today for a specific service
pub async fn get_today_metrics(
    pool: &PgPool,
    service_id: Uuid,
) -> Result<Option<ServiceMetric>, sqlx::Error> {
    let today = Utc::now().date_naive();
    
    let row = sqlx::query!(
        r#"
        SELECT id, service_id, date, uptime_percentage, average_latency_ms, 
               total_checks, successful_checks, created_at, updated_at
        FROM service_metrics 
        WHERE service_id = $1 AND date = $2
        "#,
        service_id,
        today
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| ServiceMetric {
        id: row.id,
        service_id: row.service_id,
        date: row.date,
        uptime_percentage: row.uptime_percentage.to_f64().unwrap_or(0.0),
        average_latency_ms: row.average_latency_ms as u32,
        total_checks: row.total_checks as u32,
        successful_checks: row.successful_checks as u32,
        created_at: row.created_at.and_utc(),
        updated_at: row.updated_at.and_utc(),
    }))
}

/// Delete old metrics (cleanup function)
pub async fn delete_old_metrics(
    pool: &PgPool,
    older_than_days: u32,
) -> Result<u64, sqlx::Error> {
    let cutoff_date = Utc::now().date_naive() - chrono::Duration::days(older_than_days as i64);
    
    let result = sqlx::query!(
        "DELETE FROM service_metrics WHERE date < $1",
        cutoff_date
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
} 