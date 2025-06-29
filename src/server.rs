use std::time::Duration;

use axum::{extract::State, routing::{delete, post}, Json, Router};
use http::StatusCode;
use sqlx::PgPool;
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info, Level};

use crate::{api::{CreateServiceRequest, DeleteServiceRequest}, service};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub async fn create_server(state: AppState) -> Router {
    Router::new()
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
        .with_state(state)
}

pub async fn start_server(app: Router) -> Result<(), anyhow::Error> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server started on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;
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