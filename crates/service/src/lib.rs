pub mod db;
pub mod fixtures;

use rstat_core::Service;
use sqlx::PgPool;

/// Create a new service in the database
pub async fn create(
    pool: &PgPool,
    name: &str,
    kind: rstat_core::Kind,
    interval: std::time::Duration,
) -> Result<uuid::Uuid, anyhow::Error> {
    db::create(pool, name, kind, interval).await.map_err(|e| anyhow::anyhow!(e))
}

/// Get all services from the database
pub async fn all(pool: &PgPool) -> Result<Vec<Service>, anyhow::Error> {
    db::all(pool).await.map_err(|e| anyhow::anyhow!(e))
}

/// Delete a service from the database
pub async fn delete(pool: &PgPool, id: uuid::Uuid) -> Result<(), anyhow::Error> {
    db::delete(pool, id).await
} 

pub async fn get_by_service_type(pool: &PgPool, kind: &rstat_core::Kind) -> Result<Vec<Service>, anyhow::Error> {
    db::get_by_service_type(pool, kind).await.map_err(|e| anyhow::anyhow!(e))
}