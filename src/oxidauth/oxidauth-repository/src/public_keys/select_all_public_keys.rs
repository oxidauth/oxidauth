pub use oxidauth_kernel::public_keys::{PublicKey, list_all_public_keys::ListAllPublicKeys};

use crate::prelude::*;

#[async_trait]
pub trait SelectAllPublicKeysQuery: Send + Sync + 'static {
    async fn select_all_public_keys(
        &self,
        params: &ListAllPublicKeys,
    ) -> Result<Vec<PublicKey>, BoxedError>;
}
