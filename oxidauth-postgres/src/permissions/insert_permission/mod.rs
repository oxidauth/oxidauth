use oxidauth_kernel::{error::BoxedError, permissions::create_permission::CreatePermission};
use oxidauth_repository::permissions::insert_permission::*;

use crate::prelude::*;

use super::PermissionRow;

#[async_trait]
impl<'a> Service<&'a str> for Database {
    type Response = Permission;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_user_query", skip(self))]
    async fn call(&self, params: &'a str) -> Result<Permission, BoxedError> {
        let params: CreatePermission = params.try_into()?;

        let result = sqlx::query_as::<_, PermissionRow>(include_str!(
            "insert_permission.sql"
        ))
        .bind(&params.id)
        .bind(&params.realm)
        .bind(&params.resource)
        .bind(&params.action)
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
    async fn it_should_insert_a_permission_by_id_successfully(pool: PgPool) {}
}
