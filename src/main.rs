use std::{env, path::Path};

use clap::Parser;
use sqlx::migrate::Migrator;
use tokio::sync::mpsc;
use tracing::info;

use tracing_subscriber::EnvFilter;

mod api;
mod cli;
mod healthcheck;
mod notifier;
mod scheduler;
mod server;
mod service;
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("rstat=info,tower_http=debug"))
                .unwrap(),
        )
        .compact()
        .init();
    dotenv::dotenv().ok();

    let cli = cli::Cli::parse();
    match &cli.command {
        cli::Commands::Seed => seed().await?,
        cli::Commands::Start => start().await?,
    }

    Ok(())
}

async fn start() -> Result<(), anyhow::Error> {
    let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL in the environment");

    let pool = sqlx::PgPool::connect(&database_url).await?;
    Migrator::new(Path::new("migrations"))
        .await?
        .run(&pool)
        .await?;

    let (result_tx, result_rx) = mpsc::channel(100);
    
    // Create app state
    let state = server::AppState { pool: pool.clone() };
    
    // Start scheduler
    let scheduler_handle = tokio::spawn({
        let cloned_db = pool.clone();
        let cloned_tx = result_tx.clone();
        async move {
            scheduler::start_scheduler(cloned_db, cloned_tx).await
        }
    });
    
    // Start notifier
    let notifier_handle = tokio::spawn(async move {
        notifier::start_notifier(result_rx).await;
    });

    // Start server
    let server_handle = tokio::spawn(async move {
        let app = server::create_server(state).await;
        server::start_server(app).await
    });
    
    // Wait for all components to complete
    let (scheduler_result, server_result, notifier_result) = tokio::join!(
        scheduler_handle,
        server_handle,
        notifier_handle
    );
    
    scheduler_result??;
    server_result??;
    notifier_result?;

    Ok(())
}

async fn seed() -> Result<(), anyhow::Error> {
    let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL in the environment");

    let pool = sqlx::PgPool::connect(&database_url).await?;
    Migrator::new(Path::new("migrations"))
        .await?
        .run(&pool)
        .await?;

    let svc = service::fixtures::fixtures()?;
    service::db::bulk_create(&pool, &svc).await?;

    info!("Seeding complete");
    Ok(())
}
