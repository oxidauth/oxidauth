pub use oxidauth_kernel::authorities::Authority;
use oxidauth_kernel::authorities::find_authority_by_client_key::FindAuthorityByClientKey;

pub use crate::prelude::*;

#[async_trait]
pub trait SelectAuthorityByClientKeyQuery: Send + Sync + 'static {
    async fn select_authority_by_client_key(
        &self,
        params: &FindAuthorityByClientKey,
    ) -> Result<Option<Authority>, BoxedError>;
}
