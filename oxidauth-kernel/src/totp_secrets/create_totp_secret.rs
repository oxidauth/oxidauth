use crate::dev_prelude::*;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTotpSecret {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTotpSecretResponse {
    pub success: bool,
}

#[async_trait]
pub trait CreateTotpSecretTrait: Send + Sync + 'static {
    async fn create_totp_secret(
        &self,
        params: &CreateTotpSecret,
    ) -> Result<CreateTotpSecretResponse, BoxedError>;
}

pub type CreateTotpSecretService = Arc<dyn CreateTotpSecretTrait>;
