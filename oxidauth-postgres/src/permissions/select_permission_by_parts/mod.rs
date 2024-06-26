use oxidauth_kernel::permissions::{
    find_permission_by_parts::FindPermissionByParts, RawPermission,
};
use oxidauth_repository::permissions::select_permission_by_parts::*;

use crate::prelude::*;

use super::PgPermission;

#[async_trait]
impl<'a> Service<&'a FindPermissionByParts> for Database {
    type Response = Option<Permission>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_permission_by_parts_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a FindPermissionByParts,
    ) -> Result<Option<Permission>, BoxedError> {
        let perm_string = &params.permission;
        let permission: RawPermission = perm_string.try_into()?;

        let result = sqlx::query_as::<_, PgPermission>(include_str!(
            "./query_permission_by_parts.sql"
        ))
        .bind(&permission.realm)
        .bind(&permission.resource)
        .bind(&permission.action)
        .fetch_optional(&self.pool)
        .await?;

        let permission = result.map(Into::into);

        Ok(permission)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_a_permission_by_parts_successfully(_pool: PgPool) {
    }
}
