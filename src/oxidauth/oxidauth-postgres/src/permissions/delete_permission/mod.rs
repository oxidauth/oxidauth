use oxidauth_kernel::{
    error::BoxedError,
    permissions::{delete_permission::DeletePermission, RawPermission},
};
use oxidauth_repository::permissions::delete_permission::*;

use crate::prelude::*;

use super::PgPermission;

#[async_trait]
impl<'a> Service<&'a DeletePermission> for Database {
    type Response = Permission;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_permission_query", skip(self))]
    async fn call(
        &self,
        params: &'a DeletePermission,
    ) -> Result<Permission, BoxedError> {
        let permission: RawPermission = (&params.permission).try_into()?;

        let result = sqlx::query_as::<_, PgPermission>(include_str!(
            "delete_permission.sql"
        ))
        .bind(&permission.realm)
        .bind(&permission.resource)
        .bind(&permission.action)
        .fetch_one(&self.write_pool())
        .await?;

        let permission = result.into();

        Ok(permission)
    }
}



