use crate::dev_prelude::*;

pub use super::PrivateKey;

#[async_trait]
pub trait FindMostRecentPrivateKeyTrait: Send + Sync + 'static {
    async fn find_most_recent_private_key(
        &self,
        params: &FindMostRecentPrivateKey,
    ) -> Result<PrivateKey, BoxedError>;
}

pub type FindMostRecentPrivateKeyService = Arc<dyn FindMostRecentPrivateKeyTrait>;

#[derive(Debug)]
pub struct FindMostRecentPrivateKey {}
