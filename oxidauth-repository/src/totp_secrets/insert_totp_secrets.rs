pub use oxidauth_kernel::service::Service;

pub use crate::prelude::*;

pub struct InsertTotpSecretsParams {
    pub user_id_and_secrets: Vec<(Uuid, String)>,
}

#[async_trait]
pub trait InsertTotpSecretsQuery: Send + Sync + 'static {
    async fn insert_totp_secrets(
        &self,
        params: &InsertTotpSecretsParams,
    ) -> Result<(), BoxedError>;
}
