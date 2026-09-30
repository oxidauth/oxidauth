use uuid::Uuid;

use crate::dev_prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTotpSecrets {
    pub authority_id: Uuid,
}

#[async_trait]
pub trait CreateTotpSecretsTrait: Send + Sync + 'static {
    async fn create_totp_secrets(&self, params: &CreateTotpSecrets) -> Result<(), BoxedError>;
}

pub type CreateTotpSecretsService = Arc<dyn CreateTotpSecretsTrait>;
