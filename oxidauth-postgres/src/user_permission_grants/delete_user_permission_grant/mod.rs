use crate::Database;

use oxidauth_kernel::{
    error::BoxedError,
    user_permission_grants::{
        delete_user_permission_grant::DeleteUserPermissionGrant,
        UserPermissionGrant,
    },
};
use oxidauth_repository::user_permission_grants::delete_user_permission_grant::*;

use super::PgUserPermissionGrant;

#[async_trait]
impl<'a> Service<&'a DeleteUserPermissionGrant> for Database {
    type Response = UserPermissionGrant;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "delete_user_permission_grant_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a DeleteUserPermissionGrant,
    ) -> Result<Self::Response, Self::Error> {
        let row = sqlx::query_as::<_, PgUserPermissionGrant>(include_str!(
            "./delete_user_permission_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.permission_id)
        .fetch_one(&self.pool)
        .await?;

        let user_permission_grant = row.into();

        Ok(user_permission_grant)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_be_able_to_delete_an_existing_user_permission_grant(
        pool: PgPool,
    ) {
    }
}
