use std::{env, path::Path};

use clap::Parser;
use sqlx::migrate::Migrator;
use tokio::sync::mpsc;
use tracing::info;

use tracing_subscriber::EnvFilter;

use rstat_cli::{Cli, Commands, MetricsCommands, ConfigCommands};
use rstat_api::{create_server, AppState};
use rstat_scheduler;
use rstat_seeder::Seeder;
use rstat_metrics::MetricsCalculator;
use rstat_scheduler::metrics_updater::MetricsUpdater;
use rstat_config::ConfigLoader;

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

    let cli = Cli::parse();
    match &cli.command {
        Commands::Seed => seed().await?,
        Commands::Start => start().await?,
        Commands::Config { command } => handle_config_command(command).await?,
        Commands::Metrics { command } => handle_metrics_command(command).await?,
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

    // Load services from YAML configuration on startup
    let config_loader = ConfigLoader::new(pool.clone());
    match config_loader.load_from_env().await {
        Ok(ids) => {
            if !ids.is_empty() {
                info!("Loaded {} services from environment configuration", ids.len());
            }
        }
        Err(e) => {
            info!("No environment configuration found or error loading: {}", e);
        }
    }
    
    // Try loading from default locations if no environment config
    match config_loader.load_from_default().await {
        Ok(ids) => {
            if !ids.is_empty() {
                info!("Loaded {} services from default configuration", ids.len());
            }
        }
        Err(e) => {
            info!("No default configuration found or error loading: {}", e);
        }
    }

    let (result_tx, result_rx) = mpsc::channel(100);
    
    // Create app state
    let state = AppState { pool: pool.clone() };
    
    // Start scheduler
    let scheduler_handle = tokio::spawn({
        let cloned_db = pool.clone();
        let cloned_tx = result_tx.clone();
        async move {
            rstat_scheduler::start_scheduler(cloned_db, cloned_tx).await
        }
    });
    
    // Start notifier
    let notifier_handle = tokio::spawn(async move {
        start_notifier(result_rx).await;
    });

    // Start server
    let server_handle = tokio::spawn(async move {
        let app = create_server(state).await;
        start_server(app).await
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

    let seeder = Seeder::new(pool);
    seeder.seed_all().await?;

    info!("Seeding complete");
    Ok(())
}

async fn handle_metrics_command(command: &MetricsCommands) -> Result<(), anyhow::Error> {
    let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL in the environment");
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    // Run migrations to ensure metrics table exists
    Migrator::new(Path::new("migrations"))
        .await?
        .run(&pool)
        .await?;

    let calculator = MetricsCalculator::new(pool.clone());
    let updater = MetricsUpdater::new(
        calculator,
        std::time::Duration::from_secs(300),
    );

    match command {
        MetricsCommands::Calculate => {
            info!("Calculating metrics for all services...");
            updater.update_all_metrics().await?;
            info!("Metrics calculation complete");
        }
        MetricsCommands::CalculateService { service_id } => {
            let service_uuid = uuid::Uuid::parse_str(service_id)?;
            info!("Calculating metrics for service {}...", service_id);
            updater.update_service_metrics(service_uuid).await?;
            info!("Metrics calculation complete for service {}", service_id);
        }
        MetricsCommands::CalculateYesterday => {
            info!("Calculating yesterday's metrics for all services...");
            updater.calculate_yesterday_metrics().await?;
            info!("Yesterday's metrics calculation complete");
        }
        MetricsCommands::Cleanup { days } => {
            info!("Cleaning up metrics older than {} days...", days);
            let deleted_count = updater.cleanup_old_metrics(*days).await?;
            info!("Cleaned up {} old metric records", deleted_count);
        }
    }

    Ok(())
}

async fn handle_config_command(command: &ConfigCommands) -> Result<(), anyhow::Error> {
    let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL in the environment");
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    // Run migrations to ensure services table exists
    Migrator::new(Path::new("migrations"))
        .await?
        .run(&pool)
        .await?;

    let config_loader = ConfigLoader::new(pool);

    match command {
        ConfigCommands::Load { file, skip_duplicates } => {
            info!("Loading services from file: {}", file);
            let path = Path::new(file);
            let result = if *skip_duplicates {
                config_loader.load_from_file_with_check(path).await
            } else {
                config_loader.load_from_file(path).await
            };
            
            match result {
                Ok(ids) => {
                    info!("Successfully loaded {} services from {}", ids.len(), file);
                    for id in ids {
                        println!("Created service with ID: {}", id);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load services from {}: {}", file, e);
                    return Err(e);
                }
            }
        }
        ConfigCommands::LoadDir { dir } => {
            info!("Loading services from directory: {}", dir);
            let path = Path::new(dir);
            match config_loader.load_from_directory(path).await {
                Ok(ids) => {
                    info!("Successfully loaded {} services from directory {}", ids.len(), dir);
                    for id in ids {
                        println!("Created service with ID: {}", id);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load services from directory {}: {}", dir, e);
                    return Err(e);
                }
            }
        }
        ConfigCommands::LoadDefault => {
            info!("Loading services from default configuration locations");
            
            // Try environment config first
            match config_loader.load_from_env().await {
                Ok(ids) => {
                    if !ids.is_empty() {
                        info!("Loaded {} services from environment configuration", ids.len());
                        for id in ids {
                            println!("Created service with ID: {}", id);
                        }
                    }
                }
                Err(e) => {
                    info!("No environment configuration found: {}", e);
                }
            }
            
            // Try default file
            match config_loader.load_from_default().await {
                Ok(ids) => {
                    if !ids.is_empty() {
                        info!("Loaded {} services from default configuration", ids.len());
                        for id in ids {
                            println!("Created service with ID: {}", id);
                        }
                    }
                }
                Err(e) => {
                    info!("No default configuration found: {}", e);
                }
            }
        }
    }

    Ok(())
}

async fn start_notifier(mut result_rx: tokio::sync::mpsc::Receiver<String>) {
    while let Some(result) = result_rx.recv().await {
        info!("Notification: {}", result);
    }
}

async fn start_server(app: axum::Router) -> Result<(), anyhow::Error> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Server started on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;
    Ok(())
} 