use crate::dev_prelude::*;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTotpSecretResponse {
    pub success: bool,
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
