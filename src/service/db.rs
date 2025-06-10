use std::time::Duration;


use tracing::info;

use crate::healthcheck::{self, Kind};

use super::Service;


pub async fn get(pool: &sqlx::PgPool, id: uuid::Uuid) -> Result<Service, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT id, name, kind, interval, config FROM services WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await?;
    
    
    let kind = match record.config {
        Some(config) => {
            let cfg: healthcheck::Kind = serde_json::from_value(config).unwrap();
            Some(cfg)
        }
        None => None
    }.unwrap();
    
    match kind.clone() {
        Kind::HTTP(httpchecker) => info!("HTTP checker: {:?}", httpchecker),
        Kind::TCP(tcpchecker) => info!("TCP checker: {:?}", tcpchecker),
    }
    
    let svc = Service {
        id,
        name: record.name,
        kind,
        interval: Duration::from_secs(record.interval as u64),
        
    };

    Ok(svc)
}

pub async fn create(pool: &sqlx::PgPool, name: &str, kind: Kind, interval: Duration) -> Result<uuid::Uuid, sqlx::Error> {
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