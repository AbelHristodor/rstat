use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{info, error};
use futures::future::join_all;

use crate::{
    healthcheck::{Kind, HealthCheckResult},
    service::Service,
};

pub mod data_generator;

pub struct Seeder {
    pool: PgPool,
}

impl Seeder {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn seed_all(&self) -> Result<(), anyhow::Error> {
        info!("Starting comprehensive seeding...");
        
        // Clear existing data
        self.clear_existing_data().await?;
        
        // Seed services
        let services = self.seed_services().await?;
        
        // Seed health check results for the last 30 days (concurrent)
        self.seed_health_check_results_concurrent(&services).await?;
        
        // Calculate and seed metrics (concurrent)
        self.seed_metrics_concurrent(&services).await?;
        
        info!("Seeding complete!");
        Ok(())
    }

    async fn clear_existing_data(&self) -> Result<(), anyhow::Error> {
        info!("Clearing existing data...");
        
        sqlx::query!("DELETE FROM service_metrics").execute(&self.pool).await?;
        sqlx::query!("DELETE FROM healthcheck_results").execute(&self.pool).await?;
        sqlx::query!("DELETE FROM services").execute(&self.pool).await?;
        
        info!("Existing data cleared");
        Ok(())
    }

    async fn seed_services(&self) -> Result<Vec<Service>, anyhow::Error> {
        info!("Seeding services...");
        
        let services = data_generator::generate_services();
        
        for service in &services {
            let interval_secs = service.interval.as_secs();
            let kind_str: String = service.kind.clone().into();
            let config = serde_json::to_value(service.kind.clone()).unwrap();

            match sqlx::query!(
                "INSERT INTO services (id, name, kind, interval, config) VALUES ($1, $2, $3, $4, $5)",
                service.id,
                service.name,
                kind_str,
                interval_secs as i64,
                config
            )
            .execute(&self.pool)
            .await {
                Ok(_) => info!("Seeded service: {}", service.name),
                Err(e) => {
                    error!("Failed to seed service {}: {}", service.name, e);
                    return Err(e.into());
                }
            }
        }
        
        info!("{} services seeded", services.len());
        Ok(services)
    }

    async fn seed_health_check_results_concurrent(&self, services: &[Service]) -> Result<(), anyhow::Error> {
        info!("Seeding health check results (concurrent)...");
        let pool = self.pool.clone();
        let end_date = Utc::now();
        let start_date = end_date - Duration::days(30);

        let tasks = services.iter().map(|service| {
            let pool = pool.clone();
            let service = service.clone();
            tokio::spawn(async move {
                let results = data_generator::generate_health_check_results(
                    service.id,
                    start_date,
                    end_date,
                    service.interval,
                );
                for result in results {
                    let _ = sqlx::query!(
                        "INSERT INTO healthcheck_results (id, success, code, response_time, service_id, message, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                        result.id,
                        result.success,
                        result.code as i64,
                        result.response_time as i64,
                        service.id,
                        result.message,
                        result.created_at.naive_utc()
                    )
                    .execute(&pool)
                    .await;
                }
            })
        });
        join_all(tasks).await;
        info!("Health check results seeded (concurrent)");
        Ok(())
    }

    async fn seed_metrics_concurrent(&self, services: &[Service]) -> Result<(), anyhow::Error> {
        info!("Calculating and seeding metrics (concurrent)...");
        let pool = self.pool.clone();
        let end_date = Utc::now().date_naive();
        let start_date = end_date - Duration::days(30);

        let tasks = services.iter().map(|service| {
            let pool = pool.clone();
            let service = service.clone();
            tokio::spawn(async move {
                let calculator = crate::service::metrics_calculator::MetricsCalculator::new(pool);
                let mut current_date = start_date;
                while current_date <= end_date {
                    let _ = calculator.calculate_daily_metrics(service.id, current_date).await;
                    current_date = current_date + Duration::days(1);
                }
            })
        });
        join_all(tasks).await;
        info!("Metrics calculated and seeded (concurrent)");
        Ok(())
    }
} 