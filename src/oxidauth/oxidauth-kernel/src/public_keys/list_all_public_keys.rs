pub use super::PublicKey;
use crate::dev_prelude::*;

#[async_trait]
pub trait ListAllPublicKeysServiceTrait: Send + Sync + 'static {
    async fn list_all_public_keys(
        &self,
        params: &ListAllPublicKeys,
    ) -> Result<Vec<PublicKey>, BoxedError>;
}

pub type ListAllPublicKeysService = Arc<dyn ListAllPublicKeysServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllPublicKeys;
