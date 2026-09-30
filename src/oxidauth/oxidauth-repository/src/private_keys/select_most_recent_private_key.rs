pub use oxidauth_kernel::private_keys::PrivateKey;
use oxidauth_kernel::private_keys::find_most_recent_private_key::FindMostRecentPrivateKey;

use crate::prelude::*;

#[async_trait]
pub trait SelectMostRecentPrivateKeyQuery: Send + Sync + 'static {
    async fn select_most_recent_private_key(
        &self,
        params: &FindMostRecentPrivateKey,
    ) -> Result<PrivateKey, BoxedError>;
}
#[derive(Debug)]
pub struct SelectMostRecentPrivateKey {}
