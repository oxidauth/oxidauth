pub use super::PublicKey;
use crate::dev_prelude::*;

#[async_trait]
pub trait FindPublicKeyByIdServiceTrait: Send + Sync + 'static {
    async fn find_public_key_by_id(
        &self,
        params: &FindPublicKeyById,
    ) -> Result<PublicKey, BoxedError>;
}

pub type FindPublicKeyByIdService = Arc<dyn FindPublicKeyByIdServiceTrait>;

#[derive(Debug, Deserialize)]
pub struct FindPublicKeyById {
    pub public_key_id: Uuid,
}
