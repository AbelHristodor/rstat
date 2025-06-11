use std::{collections::HashMap, time::Duration};

use futures::StreamExt;
use tokio_util::time::delay_queue;
use tracing::{debug, error, info};

use crate::{healthcheck::{self, HealthCheckRequest, HealthChecker}, service::{self}};

pub struct Scheduler {
    pub queue: tokio_util::time::DelayQueue<uuid::Uuid>,
    pub db: sqlx::PgPool,
    
    services: Vec<service::Service>,
    entries: HashMap<uuid::Uuid, (HealthCheckRequest, delay_queue::Key)>,
}

impl Scheduler {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self {
            queue: tokio_util::time::DelayQueue::new(),
            entries: HashMap::new(),
            services: Vec::new(),
            db,
        }
    }
    
    pub async fn init(mut self) -> Result<Self, anyhow::Error>{
        self.services = service::db::all(&self.db).await?;
        self.enqueue_all();
        Ok(self)
    }
    
    fn enqueue_all(&mut self) {
        for svc in self.services.clone() {
            let request = HealthCheckRequest::new(svc.clone());
            self.enqueue(request);
        }
    }

    pub fn enqueue(&mut self, request: HealthCheckRequest) {
        let timeout = request.service.interval;
        let id = request.service.id;
        let key = self.queue.insert(id, timeout);
        self.entries.insert(id, (request, key));
        info!("Enqueued service {}", id);
    }

    pub fn dequeue(&mut self, id: uuid::Uuid) {
        if let Some((_, key)) = self.entries.remove(&id) {
            self.queue.remove(&key);
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

                    if let Some((request, _)) = self.entries.get(&id) {
                        // we need to enqueue it again and process it
                        let req = request.clone();

                        // Then we run it
                        let healtcheck = match req.service.kind {
                            crate::healthcheck::Kind::HTTP(httpchecker) => {
                                httpchecker.check().await
                            }
                            crate::healthcheck::Kind::TCP(tcpchecker) => tcpchecker.check().await,
                        };
                        match healtcheck {
                            Ok(result) => {
                                debug!("Healthcheck successful for id: {}", result.id);
                                match healthcheck::db::result::create(&self.db, result,  req.service.id).await {
                                    Ok(id) => info!("Healthcheck result created with id: {}", id),
                                    Err(err) => error!("Cannot save healthcheck result to db with id: {} and err: {}", id, err),
                                }
                            },
                            Err(err) => {
                                error!("Healthcheck failed with error: {}", err);
                            }
                        };
                        

                        // enqueue it again
                        self.enqueue(request.clone())
                    }
                }
                None => {
                    info!("No health check request found");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }
}
