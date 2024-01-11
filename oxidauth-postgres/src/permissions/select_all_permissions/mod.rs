use oxidauth_kernel::permissions::list_all_permissions::ListAllPermissions;
use oxidauth_repository::permissions::select_all_permissions::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a ListAllPermissions> for Database {
    type Response = Vec<Permission>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_all_permissions_query", skip(self))]
    async fn call(&self, _params: &'a ListAllPermissions) -> Result<Vec<Permission>, BoxedError> {
        let result =
            sqlx::query_as::<_, PermissionRow>(include_str!("./select_all_permissions.sql"))
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(Into::into)
                .collect();

        Ok(result)
    }
}
