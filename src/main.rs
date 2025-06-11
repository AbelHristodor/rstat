use std::{env, path::Path, process::exit};

use anyhow::Ok;
use clap::Parser;
use sqlx::migrate::Migrator;
use tracing::info;


mod healthcheck;
mod service;
mod scheduler;
mod utils;
mod cli;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
    
    let cli = cli::Cli::parse();
    match &cli.command {
        cli::Commands::Seed=> seed().await?,
        cli::Commands::Start => start().await?,
    }

    Ok(())
}


async fn start() -> Result<(), anyhow::Error> {
    let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL in the environment"); 

    let pool = sqlx::PgPool::connect(&database_url).await?;
    Migrator::new(Path::new("migrations")).await?.run(&pool).await?;

    let cloned_db = pool.clone();
    let handle = tokio::spawn(async {
        let mut scheduler = scheduler::Scheduler::new(cloned_db).init().await.expect("Failed to initialize scheduler");
        scheduler.start().await;
    });
    handle.await?;
    Ok(())
}

async fn seed() -> Result<(), anyhow::Error> {
    let database_url = env::var("DATABASE_URL").expect("Expected DATABASE_URL in the environment"); 

    let pool = sqlx::PgPool::connect(&database_url).await?;
    Migrator::new(Path::new("migrations")).await?.run(&pool).await?;
    
    let svc = service::fixtures::fixtures()?;
    service::db::bulk_create(&pool, &svc).await?;
    
    info!("Seeding complete");
    Ok(())
}
