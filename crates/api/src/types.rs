use serde::{Deserialize, Serialize};
use rstat_core::Kind;

#[derive(Deserialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub kind: Kind,
    pub interval: u64,
}

#[derive(Deserialize)]
pub struct DeleteServiceRequest {
    pub id: String,
}

#[derive(Deserialize)]
pub struct GetServiceRequest {
    pub id: String,
} 