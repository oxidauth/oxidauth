use oxidauth_kernel::public_keys::{PublicKey, delete_public_key::DeletePublicKey};

use crate::prelude::*;

#[async_trait]
pub trait DeletePublicKeyQuery: Send + Sync + 'static {
    async fn delete_public_key(&self, params: &DeletePublicKey) -> Result<PublicKey, BoxedError>;
}
