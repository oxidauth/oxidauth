use crate::dev_prelude::*;

pub use super::PublicKey;

#[async_trait]
pub trait ListAllPublicKeysTrait: Send + Sync + 'static {
    async fn list_all_public_keys(
        &self,
        params: &ListAllPublicKeys,
    ) -> Result<Vec<PublicKey>, BoxedError>;
}

pub type ListAllPublicKeysService = Arc<dyn ListAllPublicKeysTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllPublicKeys;
