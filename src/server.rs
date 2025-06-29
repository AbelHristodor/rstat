use std::time::Duration;

use axum::{extract::State, routing::{get}, Json, Router, extract::Path};
use http::StatusCode;
use sqlx::PgPool;
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info, Level};

use crate::{api::{CreateServiceRequest, DeleteServiceRequest}, healthcheck, service};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub async fn create_server(state: AppState) -> Router {
    Router::new()
        .route("/http", 
            get(list_http_checks)
                .post(create_http_check)
                .delete(delete_http_check)
        )
        .route("/http/checks/{id}", get(get_checks_for_service))
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
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
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

async fn list_http_checks(
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<service::Service>>) {
    let checks = service::db::all(&state.pool).await;
    match checks {
        Ok(checks) => (StatusCode::OK, Json(checks)),
        Err(err) => {
            error!("Failed to list services: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

async fn get_checks_for_service(
    State(state): State<AppState>,
    Path(service_id): Path<uuid::Uuid>,
) -> (StatusCode, Json<Vec<healthcheck::HealthCheckResult>>) {
    let checks = healthcheck::db::result::get_by_service_id(&state.pool, service_id).await;
    match checks {
        Ok(checks) => (StatusCode::OK, Json(checks)),
        Err(err) => {
            error!("Failed to get checks for service: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}