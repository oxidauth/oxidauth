use oxidauth_kernel::totp_secrets::create_totp_secret::CreateTotpSecretResponse;

pub use crate::prelude::*;

pub struct InsertTotpSecretParams {
    pub user_id: Uuid,
    pub secret_key: String,
}

#[async_trait]
pub trait InsertTotpSecretQuery: Send + Sync + 'static {
    async fn insert_totp_secret(
        &self,
        params: &InsertTotpSecretParams,
    ) -> Result<CreateTotpSecretResponse, BoxedError>;
}
