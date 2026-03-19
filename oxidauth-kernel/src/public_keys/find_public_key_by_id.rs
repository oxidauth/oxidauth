use crate::dev_prelude::*;

pub use super::PublicKey;

#[async_trait]
pub trait FindPublicKeyByIdTrait: Send + Sync + 'static {
    async fn find_public_key_by_id(
        &self,
        params: &FindPublicKeyById,
    ) -> Result<PublicKey, BoxedError>;
}

pub type FindPublicKeyByIdService = Arc<dyn FindPublicKeyByIdTrait>;

#[derive(Debug, Deserialize)]
pub struct FindPublicKeyById {
    pub public_key_id: Uuid,
}
