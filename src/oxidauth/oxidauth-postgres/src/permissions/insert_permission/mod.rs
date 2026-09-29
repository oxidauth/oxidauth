use oxidauth_kernel::{
    error::BoxedError,
    permissions::{create_permission::CreatePermission, RawPermission},
};
use oxidauth_repository::permissions::insert_permission::*;

use crate::prelude::*;

use super::PgPermission;

#[async_trait]
impl<'a> Service<&'a CreatePermission> for Database {
    type Response = Permission;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_permission_query", skip(self))]
    async fn call(
        &self,
        params: &'a CreatePermission,
    ) -> Result<Permission, BoxedError> {
        let perm_string = &params.permission;
        let permission: RawPermission = perm_string.try_into()?;

        let result = sqlx::query_as::<_, PgPermission>(include_str!(
            "insert_permission.sql"
        ))
        .bind(None::<Uuid>)
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

    #[ignore]
    #[sqlx::test]
    async fn it_should_insert_a_permission_by_id_successfully(_pool: PgPool) {}
}
