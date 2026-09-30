pub use super::PublicKey;
use crate::dev_prelude::*;

#[async_trait]
pub trait CreatePublicKeyServiceTrait: Send + Sync + 'static {
    async fn create_public_key(&self, params: &CreatePublicKey) -> Result<PublicKey, BoxedError>;
}

pub type CreatePublicKeyService = Arc<dyn CreatePublicKeyServiceTrait>;

#[derive(Debug)]
pub struct CreatePublicKey;
