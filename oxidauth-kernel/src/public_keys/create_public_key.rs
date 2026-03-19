use crate::dev_prelude::*;

pub use super::PublicKey;

#[async_trait]
pub trait CreatePublicKeyTrait: Send + Sync + 'static {
    async fn create_public_key(
        &self,
        params: &CreatePublicKey,
    ) -> Result<PublicKey, BoxedError>;
}

pub type CreatePublicKeyService = Arc<dyn CreatePublicKeyTrait>;

#[derive(Debug)]
pub struct CreatePublicKey;
