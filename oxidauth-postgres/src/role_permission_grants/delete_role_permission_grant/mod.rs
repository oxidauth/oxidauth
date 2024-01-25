use oxidauth_repository::role_permission_grants::delete_role_permission_grant::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a DeleteRolePermissionGrantParams> for Database {
    type Response = RolePermissionGrant;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "delete_role_permission_grant_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a DeleteRolePermissionGrantParams,
    ) -> Result<RolePermissionGrant, BoxedError> {
        let result = sqlx::query_as::<_, PgRolePermissionGrant>(include_str!(
            "./delete_role_permission_grant.sql"
        ))
        .bind(params.role_id)
        .bind(params.permission_id)
        .fetch_one(&self.pool)
        .await?;

        let role_permission_grant = result.into();

        Ok(role_permission_grant)
    }
}
