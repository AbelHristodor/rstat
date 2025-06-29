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
mod seeder;
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
        cli::Commands::Metrics { command } => handle_metrics_command(command).await?,
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

    let seeder = seeder::Seeder::new(pool);
    seeder.seed_all().await?;

    info!("Seeding complete");
    Ok(())
}

async fn handle_metrics_command(command: &cli::MetricsCommands) -> Result<(), anyhow::Error> {
    let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL in the environment");
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    // Run migrations to ensure metrics table exists
    Migrator::new(Path::new("migrations"))
        .await?
        .run(&pool)
        .await?;

    let calculator = service::metrics_calculator::MetricsCalculator::new(pool.clone());
    let updater = scheduler::metrics_updater::MetricsUpdater::new(
        calculator,
        std::time::Duration::from_secs(300),
    );

    match command {
        cli::MetricsCommands::Calculate => {
            info!("Calculating metrics for all services...");
            updater.update_all_metrics().await?;
            info!("Metrics calculation complete");
        }
        cli::MetricsCommands::CalculateService { service_id } => {
            let service_uuid = uuid::Uuid::parse_str(service_id)?;
            info!("Calculating metrics for service {}...", service_id);
            updater.update_service_metrics(service_uuid).await?;
            info!("Metrics calculation complete for service {}", service_id);
        }
        cli::MetricsCommands::CalculateYesterday => {
            info!("Calculating yesterday's metrics for all services...");
            updater.calculate_yesterday_metrics().await?;
            info!("Yesterday's metrics calculation complete");
        }
        cli::MetricsCommands::Cleanup { days } => {
            info!("Cleaning up metrics older than {} days...", days);
            let deleted_count = updater.cleanup_old_metrics(*days).await?;
            info!("Cleaned up {} old metric records", deleted_count);
        }
    }

    Ok(())
}
