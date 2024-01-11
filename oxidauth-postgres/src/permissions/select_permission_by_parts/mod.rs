use oxidauth_kernel::permissions::find_permission_by_parts::FindPermissionByParts;
use oxidauth_repository::permissions::select_permission_by_parts::*;

use crate::prelude::*;

#[async_trait]
impl<'a> Service<&'a FindPermissionByParts> for Database {
    type Response = Permission;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_permission_by_parts_query", skip(self))]
    async fn call(&self, params: &'a FindPermissionByParts) -> Result<Permission, BoxedError> {
        let perm_string = &params.permission;
        let permission: Permission = perm_string.try_into()?;

        let result = sqlx::query_as::<_, super::PermissionRow>(include_str!(
            "./query_permission_by_parts.sql"
        ))
        .bind(&permission.realm)
        .bind(&permission.resource)
        .bind(&permission.action)
        .fetch_one(&self.pool)
        .await?;

        let permission = result.into();

        Ok(permission)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_permission_by_parts_successfully(pool: PgPool) {}
}
