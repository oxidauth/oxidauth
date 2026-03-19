use crate::dev_prelude::*;

pub use super::PublicKey;

#[async_trait]
pub trait DeletePublicKeyTrait: Send + Sync + 'static {
    async fn delete_public_key(
        &self,
        params: &DeletePublicKey,
    ) -> Result<PublicKey, BoxedError>;
}

pub type DeletePublicKeyService = Arc<dyn DeletePublicKeyTrait>;

#[derive(Debug, Deserialize)]
pub struct DeletePublicKey {
    pub public_key_id: Uuid,
}
