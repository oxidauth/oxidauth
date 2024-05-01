use crate::dev_prelude::*;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthKeyResponse {
    pub user_id: Uuid,
}

pub struct InsertAuthKeyParams {
    pub user_id: Uuid,
    pub secret_key: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: Vec<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuthKey {
    pub user_id: Uuid,
}

pub type CreateAuthKeyService = Arc<
    dyn for<'a> Service<
        &'a CreateAuthKey,
        Response = CreateAuthKeyResponse,
        Error = BoxedError,
    >,
>;
