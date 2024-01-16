pub use oxidauth_kernel::permissions::Permission;

use crate::prelude::*;

#[async_trait]
pub trait QueryPermissionsByRealm: Send + Sync + 'static {
    async fn query_permissions_by_realm(
        &self,
        realm: String,
    ) -> Result<Vec<Permission>, QueryPermissionsByRealmError>;
}

#[derive(Debug)]
pub struct QueryPermissionsByRealmError {}
