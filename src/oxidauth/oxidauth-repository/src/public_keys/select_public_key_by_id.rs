pub use oxidauth_kernel::public_keys::{PublicKey, find_public_key_by_id::FindPublicKeyById};

use crate::prelude::*;

#[async_trait]
pub trait SelectPublicKeyByIdQuery: Send + Sync + 'static {
    async fn select_public_key_by_id(
        &self,
        params: &FindPublicKeyById,
    ) -> Result<PublicKey, BoxedError>;
}
