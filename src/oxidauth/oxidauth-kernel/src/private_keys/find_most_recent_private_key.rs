pub use super::PrivateKey;
use crate::dev_prelude::*;

#[async_trait]
pub trait FindMostRecentPrivateKeyServiceTrait: Send + Sync + 'static {
    async fn find_most_recent_private_key(
        &self,
        params: &FindMostRecentPrivateKey,
    ) -> Result<PrivateKey, BoxedError>;
}

pub type FindMostRecentPrivateKeyService = Arc<dyn FindMostRecentPrivateKeyServiceTrait>;

#[derive(Debug)]
pub struct FindMostRecentPrivateKey {}
