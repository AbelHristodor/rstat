use std::{env, path::Path, time::Duration};

use sqlx::migrate::Migrator;
use tracing::info;


mod healthcheck;
mod service;
mod scheduler;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();
    // Set the DATABASE_URL as a compile-time environment variable.
    if let Ok(database_url) = env::var("DATABASE_URL") {
        println!("cargo:rerun-if-env-changed=DATABASE_URL={}", database_url);
    }

    let db_url = "postgres://postgres:postgres@localhost/postgres".to_string();
    let pool = sqlx::PgPool::connect(&db_url).await?;
    Migrator::new(Path::new("migrations")).await?.run(&pool).await?;

    let cloned_db = pool.clone();
    let handle = tokio::spawn(async {
        let mut scheduler = scheduler::Scheduler::new(cloned_db);
        scheduler.start().await;
    });

    let checker = healthcheck::http::new("localhost:5000/health".into(), None);
    let http = healthcheck::Kind::HTTP(checker);

    let id = service::db::create(&pool, "example", http.into(), Duration::from_secs(10)).await?;
    info!("Created service with id: {}", id);
    
    let svc = service::db::get(&pool, id).await?;
    info!("Retrieved service with id: {:?}", svc);

    handle.await?;

    Ok(())
}
