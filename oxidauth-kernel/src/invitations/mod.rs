use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod create_invitation;
pub mod delete_invitation;
pub mod find_invitation;

#[derive(Debug, Serialize, Deserialize)]
pub struct Invitation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub creaated_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
