use std::collections::HashMap;

use futures::StreamExt;
use tokio_util::time::delay_queue;
use tracing::{debug, error, info};

use crate::healthcheck::{HealthCheckRequest, HealthChecker};

pub struct Scheduler {
    pub queue: tokio_util::time::DelayQueue<uuid::Uuid>,
    entries: HashMap<uuid::Uuid, (HealthCheckRequest, delay_queue::Key)>
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            queue: tokio_util::time::DelayQueue::new(),
            entries: HashMap::new()
        }
    }

    pub fn enqueue(&mut self, request: HealthCheckRequest) {
        let timeout = request.service.interval;
        let id = request.service.id;
        let key = self.queue.insert(id, timeout);
        self.entries.insert(id, (request, key));
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
                        let kind: String =  req.service.kind.clone().into();
                        
                        // Then we run it
                        let result = match req.service.kind {
                            crate::healthcheck::Kind::HTTP(httpchecker) => {
                                httpchecker.check().await
                            }
                            crate::healthcheck::Kind::TCP(tcpchecker) => {
                                tcpchecker.check().await
                            }
                        };
                        
                        match result.error {
                            Some(err) => {
                                error!("Health check failed for service {} - {}: {}", req.service.name, kind, err);
                            }
                            None => {
                                info!("Health check successful for service {} - {}", req.service.name, kind);
                            }
                        }
                        
                        // enqueue it again
                        self.enqueue(request.clone())
                    }
                },
                None => continue,
            }
        }
    }
}
