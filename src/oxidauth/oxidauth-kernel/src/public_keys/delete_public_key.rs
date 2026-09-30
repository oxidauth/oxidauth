pub use super::PublicKey;
use crate::dev_prelude::*;

#[async_trait]
pub trait DeletePublicKeyServiceTrait: Send + Sync + 'static {
    async fn delete_public_key(&self, params: &DeletePublicKey) -> Result<PublicKey, BoxedError>;
}

pub type DeletePublicKeyService = Arc<dyn DeletePublicKeyServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct DeletePublicKey {
    pub public_key_id: Uuid,
}
