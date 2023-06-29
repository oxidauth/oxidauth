use crate::prelude::*;

use super::PermissionRow;

#[async_trait]
pub trait QueryPermissionsByRealm: Send + Sync + 'static {
    async fn query_permissions_by_realm(
        &self,
        realm: String,
    ) -> Result<Vec<PermissionRow>, QueryPermissionsByRealmError>;
}

#[derive(Debug)]
pub struct QueryPermissionsByRealmError {}
