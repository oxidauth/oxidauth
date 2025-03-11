use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::service::Service;

pub use super::Authority;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByClientKey {
    pub client_key: Uuid,
}
