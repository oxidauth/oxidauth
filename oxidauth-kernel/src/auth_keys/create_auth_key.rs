use crate::dev_prelude::*;
use uuid::Uuid;

pub struct InsertAuthKeyParams {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: Vec<i32>,
    pub created_at: DateTime<Utc>,
}
