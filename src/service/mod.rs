use std::time::Duration;

use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::healthcheck::Kind;

pub mod db;
pub mod fixtures;

/// Service represents an entity that can be checked for health.
#[derive(Clone, Debug)]
pub struct Service {
    /// Unique identifier for the service.
    pub id: Uuid,
    /// Name of the service.
    pub name: String,
    /// The type of healthcheck to perform.
    /// For example HTTP, GRPC, ICMP etc... 
    pub kind: Kind,
    /// The interval at which the service should be checked for health.
    pub interval: Duration,
    /// The next time this service should be checked.
    pub next_run: DateTime<Utc>,
}

impl PartialEq for Service {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}