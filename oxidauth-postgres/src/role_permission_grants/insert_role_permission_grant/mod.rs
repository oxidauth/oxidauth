use oxidauth_kernel::role_permission_grants::create_role_permission_grant::*;
use oxidauth_repository::role_permission_grants::insert_role_permission_grant::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a InsertRolePermissionGrant> for Database {
    type Response = RolePermissionGrant;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "insert_role_permission_grant_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a InsertRolePermissionGrant,
    ) -> Result<RolePermissionGrant, BoxedError> {
        let result = sqlx::query_as::<_, PgRolePermissionGrant>(include_str!(
            "./insert_role_permission_grant.sql"
        ))
        .bind(&params.role_id)
        .bind(&params.permission_id)
        .fetch_one(&self.pool)
        .await?;

        let role_permission_grant = result.into();

        Ok(role_permission_grant)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_insert_a_role_permission_grant_successfully(
        _pool: PgPool,
    ) {
    }
}
