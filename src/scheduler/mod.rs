use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::StreamExt;
use tokio::sync::{mpsc, Mutex};
use tokio_util::time::delay_queue;
use tracing::{debug, error, info, warn};

use crate::{
    healthcheck::{self, HealthCheckRequest, HealthChecker},
    service::{self, Service},
};

pub struct Scheduler {
    pub queue: tokio_util::time::DelayQueue<uuid::Uuid>,
    pub db: sqlx::PgPool,
    pub result_tx: mpsc::Sender<String>,

    entries: Arc<Mutex<HashMap<uuid::Uuid, (HealthCheckRequest, delay_queue::Key)>>>,
}

impl Scheduler {
    pub fn new(db: sqlx::PgPool, result_tx: mpsc::Sender<String>) -> Self {
        Self {
            queue: tokio_util::time::DelayQueue::new(),
            entries: Arc::new(Mutex::new(HashMap::new())),
            result_tx,
            db,
        }
    }

    pub async fn init(mut self) -> Result<Self, anyhow::Error> {
        let services = service::db::all(&self.db).await?;
        self.enqueue_all(services).await;
        Ok(self)
    }

    async fn enqueue_all(&mut self, services: Vec<Service>) {
        for svc in services {
            let request = HealthCheckRequest::new(svc.clone());
            self.enqueue(request).await;
        }
    }

    async fn enqueue(&mut self, request: HealthCheckRequest) {
        let timeout = request.service.interval;
        let id = request.service.id;
        let key = self.queue.insert(id, timeout);
        self.entries.lock().await.insert(id, (request, key));
        info!("Enqueued service {}", id);
    }

    async fn get(&self, id: uuid::Uuid) -> Option<HealthCheckRequest> {
        if let Some((request, _)) = self.entries.lock().await.get(&id) {
            Some(request.clone())
        } else {
            None
        }
    }
    
    async fn refresh_all(&mut self) {
        let entries = self.entries.lock().await.clone();
        self.queue.clear();

        for (id, (request, _)) in &entries {
            let timeout = request.service.interval;
            let key = self.queue.insert(*id, timeout);
            self.entries.lock().await.insert(*id, (request.clone(), key));
        }
    }

    pub async fn start(&mut self) {
        info!("Starting scheduler");
        loop {
            match self.queue.next().await {
                Some(_request) => {
                    debug!("Processing request");
                    // Process the request
                    let id = _request.into_inner();

                    match self.get(id).await {
                        Some(request) => {
                            // we need to enqueue it again and process it
                            let req = request.clone();

                            // Then we run it
                            let healtcheck = match req.service.kind {
                                crate::healthcheck::Kind::HTTP(httpchecker) => {
                                    httpchecker.check().await
                                }
                                crate::healthcheck::Kind::TCP(tcpchecker) => {
                                    tcpchecker.check().await
                                }
                            };
                            match healtcheck {
                                Ok(result) => {
                                    debug!("Healthcheck successful for id: {}", result.id);
                                    match healthcheck::db::result::create(
                                        &self.db,
                                        result,
                                        req.service.id,
                                    )
                                    .await
                                    {
                                        Ok(id) => {
                                            info!("Healthcheck result created with id: {}", id)
                                        }
                                        Err(err) => error!(
                                            "Cannot save healthcheck result to db with id: {} and err: {}",
                                            id, err
                                        ),
                                    }
                                    let _ = self.result_tx.send("Hello".into()).await;
                                }
                                Err(err) => {
                                    error!("Healthcheck failed with error: {}", err);
                                }
                            };
                            self.enqueue(request).await;
                        }
                        None => {
                            warn!("Mismatch between queue and entries! This should not happen!")
                        },
                    }
                }
                None => {
                    info!("No health check request found");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    self.refresh_all().await;
                }
            }
        }
    }
}
