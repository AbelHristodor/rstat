use std::time::Duration;

use tracing::{info, warn};
use chrono::{DateTime, Utc};
use rstat_core::{Service, Kind};

pub async fn all(pool: &sqlx::PgPool) -> Result<Vec<Service>, sqlx::Error> {
    let rows = sqlx::query!("SELECT id, name, kind, interval, config, next_run FROM services")
        .fetch_all(pool)
        .await?;

    let svc: Vec<Service> = rows
        .iter()
        .map(|row| {
            let kind = match &row.config {
                Some(config) => {
                    let cfg: Kind = serde_json::from_value(config.clone()).unwrap();
                    Some(cfg)
                }
                None => {
                    warn!("Service {} has no config", row.id);
                    None
                }
            }
            .unwrap();

            Service {
                id: row.id,
                name: row.name.clone(),
                kind,
                interval: Duration::from_secs(row.interval as u64),
                next_run: row.next_run.and_utc(),
            }
        })
        .collect();

    Ok(svc)
}

pub async fn get(pool: &sqlx::PgPool, id: String) -> Result<Service, anyhow::Error> {
    let uuid = uuid::Uuid::parse_str(&id)?;
    let record = sqlx::query!(
        "SELECT id, name, kind, interval, config, next_run FROM services WHERE id = $1",
        uuid
    )
    .fetch_one(pool)
    .await?;

    let kind = match record.config {
        Some(config) => {
            let cfg: Kind = serde_json::from_value(config).unwrap();
            Some(cfg)
        }
        None => None,
    }
    .unwrap();

    let svc = Service {
        id: uuid,
        name: record.name,
        kind,
        interval: Duration::from_secs(record.interval as u64),
        next_run: record.next_run.and_utc(),
    };

    Ok(svc)
}

pub async fn get_by_service_type(pool: &sqlx::PgPool, kind: &Kind) -> Result<Vec<Service>, sqlx::Error> {
    let kind_str = kind.to_string();
    let rows = sqlx::query!("SELECT id, name, kind, interval, config, next_run FROM services WHERE kind = $1", kind_str)
        .fetch_all(pool)
        .await?;
    
    let svc: Vec<Service> = rows
        .iter()
        .map(|row| {
            let kind = match &row.config {
                Some(config) => {
                    let cfg: Kind = serde_json::from_value(config.clone()).unwrap();
                    Some(cfg)
                }
                None => {
                    warn!("Service {} has no config", row.id);
                    None
                }
            }
            .unwrap();

            Service {
                id: row.id,
                name: row.name.clone(),
                kind,
                interval: Duration::from_secs(row.interval as u64),
                next_run: row.next_run.and_utc(),
            }
        })
        .collect();

    Ok(svc)
}

pub async fn create(
    pool: &sqlx::PgPool,
    name: &str,
    kind: Kind,
    interval: Duration,
) -> Result<uuid::Uuid, sqlx::Error> {
    let id = uuid::Uuid::new_v4();
    let interval_secs = interval.as_secs();
    let kind_str: String = kind.clone().into();
    let config = serde_json::to_value(kind).unwrap();

    let svc = sqlx::query!(
        "INSERT INTO services (id, name, kind, interval, config) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        id,
        name,
        kind_str,
        interval_secs as i64,
        config
    )
    .fetch_one(pool)
    .await?;

    Ok(svc.id)
}

pub async fn bulk_create(pool: &sqlx::PgPool, services: &Vec<Service>) -> Result<(), sqlx::Error> {
    for service in services {
        let id = uuid::Uuid::new_v4();
        let interval_secs = service.interval.as_secs();
        let kind_str: String = service.kind.clone().into();
        let config = serde_json::to_value(service.kind.clone()).unwrap();

        sqlx::query!(
            "INSERT INTO services (id, name, kind, interval, config) VALUES ($1, $2, $3, $4, $5)",
            id,
            service.name,
            kind_str,
            interval_secs as i64,
            config
        )
        .execute(pool)
        .await?;
        info!("Service {} created", service.name);
    }

    Ok(())
}

pub async fn delete(pool: &sqlx::PgPool, id: uuid::Uuid) -> Result<(), anyhow::Error> {
    sqlx::query!("DELETE FROM services WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_next_run(
    pool: &sqlx::PgPool,
    service_id: uuid::Uuid,
    next_run: DateTime<Utc>,
) -> Result<(), sqlx::Error> {

    sqlx::query!(
        "UPDATE services SET next_run = $1 WHERE id = $2",
        next_run.naive_utc(),
        service_id
    )
    .execute(pool)
    .await?;

    Ok(())
} 