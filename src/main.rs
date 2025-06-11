use std::{env, path::Path};

use anyhow::Ok;
use axum::{Json, Router, routing::post};
use clap::Parser;
use http::StatusCode;
use sqlx::migrate::Migrator;
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, info};

use tracing_subscriber::EnvFilter;

mod cli;
mod healthcheck;
mod scheduler;
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

    let cloned_db = pool.clone();
    let handle = tokio::spawn(async {
        let mut scheduler = scheduler::Scheduler::new(cloned_db)
            .init()
            .await
            .expect("Failed to initialize scheduler");
        scheduler.start().await;
    });

    let server = tokio::spawn(async {
        let app = Router::new().route("/http", post(create_http_check)).layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        );

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!("Server started on {}", listener.local_addr().unwrap());
        axum::serve(listener, app).await.unwrap();
    });

    handle.await?;
    server.await?;

    Ok(())
}
async fn create_http_check() -> (StatusCode, Json<String>) {
    (StatusCode::OK, Json("OK".to_string()))
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
