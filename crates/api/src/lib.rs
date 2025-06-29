use std::time::Duration;

use axum::{extract::State, routing::{get}, Json, Router, extract::Path, extract::Query};
use http::StatusCode;
use sqlx::PgPool;
use serde::Deserialize;
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    cors::{CorsLayer, Any},
};
use tracing::{error, info, Level};

use rstat_core::{Service, Kind};
use rstat_service;
use rstat_healthcheck;
use rstat_metrics::{ServiceMetric, ServiceMetricsSummary, MetricsCalculator};

pub mod types;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[derive(Deserialize)]
pub struct MetricsQuery {
    days: Option<u32>,
}

pub async fn create_server(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/http", 
            get(list_http_checks)
                .post(create_http_check)
                .delete(delete_http_check)
        )
        .route("/http/checks/{id}", get(get_checks_for_service))
        .route("/metrics", get(get_all_metrics))
        .route("/metrics/{service_id}", get(get_service_metrics))
        .route("/metrics/{service_id}/summary", get(get_service_metrics_summary))
        .layer(cors)
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

async fn create_http_check(
    State(state): State<AppState>,
    Json(payload): Json<types::CreateServiceRequest>,
) -> (StatusCode, Json<String>) {
    let svc = rstat_service::create(
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
    Json(payload): Json<types::DeleteServiceRequest>,
) -> StatusCode {
    let service_id = match uuid::Uuid::parse_str(&payload.id) {
        Ok(id) => id,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    
    let svc = rstat_service::delete(&state.pool, service_id).await;
    
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
) -> (StatusCode, Json<Vec<Service>>) {
    let checks = rstat_service::all(&state.pool).await;
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
) -> (StatusCode, Json<Vec<rstat_core::HealthCheckResult>>) {
    let checks = rstat_healthcheck::db::get_by_service_id(&state.pool, service_id).await;
    match checks {
        Ok(checks) => (StatusCode::OK, Json(checks)),
        Err(err) => {
            error!("Failed to get checks for service: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

async fn get_all_metrics(
    State(state): State<AppState>,
) -> (StatusCode, Json<Vec<ServiceMetric>>) {
    let end_date = chrono::Utc::now().date_naive();
    let start_date = end_date - chrono::Duration::days(30);
    
    let metrics = rstat_metrics::db::get_all_metrics(
        &state.pool,
        start_date,
        end_date,
    ).await;
    
    match metrics {
        Ok(metrics) => (StatusCode::OK, Json(metrics)),
        Err(err) => {
            error!("Failed to get all metrics: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

async fn get_service_metrics(
    State(state): State<AppState>,
    Path(service_id): Path<uuid::Uuid>,
    Query(query): Query<MetricsQuery>,
) -> (StatusCode, Json<Vec<ServiceMetric>>) {
    let days = query.days.unwrap_or(30);
    let metrics = rstat_metrics::db::get_metrics_for_service_last_days(
        &state.pool,
        service_id,
        days,
    ).await;
    
    match metrics {
        Ok(metrics) => (StatusCode::OK, Json(metrics)),
        Err(err) => {
            error!("Failed to get metrics for service {}: {}", service_id, err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

async fn get_service_metrics_summary(
    State(state): State<AppState>,
    Path(service_id): Path<uuid::Uuid>,
    Query(query): Query<MetricsQuery>,
) -> (StatusCode, Json<ServiceMetricsSummary>) {
    let days = query.days.unwrap_or(30);
    let calculator = MetricsCalculator::new(state.pool.clone());
    let summary = calculator.get_metrics_summary(service_id, Some(days)).await;
    
    match summary {
        Ok(summary) => (StatusCode::OK, Json(summary)),
        Err(err) => {
            error!("Failed to get metrics summary for service {}: {}", service_id, err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ServiceMetricsSummary {
                service_id,
                current_uptime: 0.0,
                current_latency_ms: 0,
                average_latency_ms: 0,
                uptime_data: vec![],
            }))
        }
    }
} 