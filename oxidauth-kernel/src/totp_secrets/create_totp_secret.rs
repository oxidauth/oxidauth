use crate::dev_prelude::*;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTotpSecretResponse {
    pub user_id: Uuid,
}

pub struct InsertTotpSecretParams {
    pub user_id: Uuid,
    pub secret_key: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpSecret {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: Vec<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTotpSecret {
    pub user_id: Uuid,
}

pub type CreateTotpSecretService = Arc<
    dyn for<'a> Service<
        &'a CreateTotpSecret,
        Response = CreateTotpSecretResponse,
        Error = BoxedError,
    >,
>;
