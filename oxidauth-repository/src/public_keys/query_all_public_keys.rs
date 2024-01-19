use oxidauth_kernel::public_keys::PublicKey;

use crate::prelude::*;

#[async_trait]
pub trait QueryAllPublicKeys: Send + Sync + 'static {
    async fn query_all_public_keys(
        &self,
    ) -> Result<Vec<PublicKey>, QueryAllPublicKeysError>;
}

#[derive(Debug)]
pub struct QueryAllPublicKeysError {}
