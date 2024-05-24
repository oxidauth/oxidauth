use crate::dev_prelude::*;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTotpSecretResponse {
    pub success: bool,
}

pub struct InsertTotpSecretParams {
    pub user_id: Uuid,
    pub secret_key: String,
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
