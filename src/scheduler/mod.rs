use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task;
use futures::future::join_all;
use tracing::{debug, error, info};
use chrono::Utc;

use crate::{
    healthcheck::{self, HealthChecker},
    service::{self, Service},
};

pub async fn start_scheduler(db: sqlx::PgPool, result_tx: mpsc::Sender<String>) -> Result<(), anyhow::Error> {
    let scheduler = Scheduler::new(db, result_tx)
        .init()
        .await
        .expect("Failed to initialize scheduler");
    
    info!("Starting scheduler");
    scheduler.start().await;
    
    Ok(())
} 

#[derive(Clone)]
pub struct Scheduler {
    pub db: sqlx::PgPool,
    pub result_tx: mpsc::Sender<String>,
}

impl Scheduler {
    pub fn new(db: sqlx::PgPool, result_tx: mpsc::Sender<String>) -> Self {
        Self {
            db,
            result_tx,
        }
    }

    pub async fn init(self) -> Result<Self, anyhow::Error> {
        info!("Scheduler initialized");
        Ok(self)
    }

    async fn get_due_services(&self) -> Result<Vec<Service>, anyhow::Error> {
        let services = service::db::all(&self.db).await?;
        let now = Utc::now();
        
        let due_services: Vec<Service> = services
            .into_iter()
            .filter(|service| service.next_run <= now)
            .collect();
        
        Ok(due_services)
    }

    async fn run_healthcheck(&self, service: &Service) {
        info!("Running healthcheck for service: {}", service.name);
        
        let healthcheck = match &service.kind {
            crate::healthcheck::Kind::HTTP(httpchecker) => {
                httpchecker.check().await
            }
            crate::healthcheck::Kind::TCP(tcpchecker) => {
                tcpchecker.check().await
            }
        };

        match healthcheck {
            Ok(result) => {
                debug!("Healthcheck successful for service: {}", service.name);
                match healthcheck::db::result::create(
                    &self.db,
                    result,
                    service.id,
                )
                .await
                {
                    Ok(id) => {
                        info!("Healthcheck result created with id: {}", id)
                    }
                    Err(err) => error!(
                        "Cannot save healthcheck result to db for service {} with err: {}",
                        service.name, err
                    ),
                }
                let _ = self.result_tx.send(format!("Healthcheck completed for {}", service.name)).await;
            }
            Err(err) => {
                error!("Healthcheck failed for service {} with error: {}", service.name, err);
            }
        }

        // Update the next_run timestamp
        let next_run = Utc::now() + chrono::Duration::from_std(service.interval).unwrap();
        if let Err(err) = service::db::update_next_run(&self.db, service.id, next_run).await {
            error!("Failed to update next_run for service {}: {}", service.name, err);
        }
    }

    pub async fn start(&self) {
        info!("Starting scheduler with periodic approach");
        
        loop {
            match self.get_due_services().await {
                Ok(due_services) => {
                    if !due_services.is_empty() {
                        info!("Found {} services due for healthcheck", due_services.len());
                        
                        let tasks: Vec<_> = due_services.into_iter().map(|service| {
                            let scheduler = self.clone();
                            task::spawn(async move {
                                scheduler.run_healthcheck(&service).await;
                            })
                        }).collect();

                        join_all(tasks).await;
                    } else {
                        debug!("No services due for healthcheck");
                    }
                }
                Err(err) => {
                    error!("Failed to get due services: {}", err);
                }
            }
            
            // Wait 2 seconds before next check
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}
