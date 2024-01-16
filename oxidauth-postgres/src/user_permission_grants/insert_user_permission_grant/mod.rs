use crate::Database;

use oxidauth_kernel::{
    error::BoxedError,
    user_permission_grants::{
        create_user_permission_grant::CreateUserPermissionGrant,
        UserPermissionGrant,
    },
};
use oxidauth_repository::user_permission_grants::insert_user_permission_grant::*;

use super::PgUserPermissionGrant;

#[async_trait]
impl<'a> Service<&'a CreateUserPermissionGrant> for Database {
    type Response = UserPermissionGrant;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "insert_user_permission_grant_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a CreateUserPermissionGrant,
    ) -> Result<Self::Response, Self::Error> {
        let row = sqlx::query_as::<_, PgUserPermissionGrant>(include_str!(
            "./insert_user_permission_grant.sql"
        ))
        .bind(&params.user_id)
        .bind(&params.permission_id)
        .fetch_one(&self.pool)
        .await?;

        let user_permission_grant = row.try_into()?;

        Ok(user_permission_grant)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_be_able_to_insert_a_new_user_permission_grant(
        pool: PgPool,
    ) {
    }
}
