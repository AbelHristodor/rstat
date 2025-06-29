use std::{env, path::Path, time::Duration};

use api::{CreateServiceRequest, DeleteServiceRequest};
use axum::{extract::State, routing::{delete, post}, Json, Router};
use clap::Parser;
use http::StatusCode;
use sqlx::migrate::Migrator;
use tokio::sync::mpsc;
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info, Level};

use tracing_subscriber::EnvFilter;

mod api;
mod cli;
mod healthcheck;
mod scheduler;
mod service;
mod utils;

#[derive(Clone)]
struct AppState {
    pool: sqlx::PgPool,
}

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
    
    let state = AppState { pool: pool.clone() };
    
    let (result_tx, mut result_rx) = mpsc::channel(100);
    
    let cloned_tx = result_tx.clone();
    let scheduler = tokio::spawn(async {
        let scheduler = scheduler::Scheduler::new(cloned_db, cloned_tx)
            .init()
            .await
            .expect("Failed to initialize scheduler");
        
        scheduler.start().await;
    });
    
    let notifier = tokio::spawn({
        async move {
            while let Some(result) = result_rx.recv().await {
                info!("Received result: {:?}", result);
            }
        }
    });

    let server = tokio::spawn(async {
        let app = Router::new()
            .route("/http", post(create_http_check))
            .route("/http", delete(delete_http_check))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!("Server started on {}", listener.local_addr().unwrap());
        axum::serve(listener, app).await.unwrap();
    });
    
    scheduler.await?;
    server.await?;
    notifier.await?;

    Ok(())
}

async fn create_http_check(
    State(state): State<AppState>,
    Json(payload): Json<CreateServiceRequest>,
) -> (StatusCode, Json<String>) {
    
    let svc = service::db::create(
        &state.pool,
        &payload.name,
        payload.kind,
        Duration::from_secs(payload.interval),
    ).await;
    
    match svc {
        Ok(id) => (StatusCode::CREATED, Json(id.to_string())),
        Err(err) => (StatusCode::BAD_REQUEST, Json(err.to_string())),
    }
}

async fn delete_http_check(
    State(state): State<AppState>,
    Json(payload): Json<DeleteServiceRequest>,
) -> StatusCode {
    
    let svc = service::db::delete(
        &state.pool,
        payload.id,
    ).await;
    
    match svc {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            error!("Failed to delete service: {}", err);
            StatusCode::BAD_REQUEST
        }
    }
}

// async fn get_http_check(
//     State(state): State<AppState>,
//     Json(payload): Json<GetServiceRequest>,
// ) -> (StatusCode, Json<Service>) {
    
//     let svc = service::db::get(
//         &state.pool,
//         payload.id,
//     ).await;
    
//     match svc {
//         Ok(svc) => (StatusCode::OK, Json(svc)),
//         Err(err) => (StatusCode::BAD_REQUEST, Json()),
//     }
// }

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
