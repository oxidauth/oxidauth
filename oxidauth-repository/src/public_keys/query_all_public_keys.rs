use crate::prelude::*;

use super::PublicKeyRow;

#[async_trait]
pub trait QueryAllPublicKeys: Send + Sync + 'static {
    async fn query_all_public_keys(&self) -> Result<Vec<PublicKeyRow>, QueryAllPublicKeysError>;
}

#[derive(Debug)]
pub struct QueryAllPublicKeysError {}
