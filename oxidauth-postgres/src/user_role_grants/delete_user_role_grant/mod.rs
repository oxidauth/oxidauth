use crate::Database;

use oxidauth_kernel::{
    error::BoxedError,
    user_role_grants::{
        delete_user_role_grant::DeleteUserRoleGrant, UserRoleGrant,
    },
};
use oxidauth_repository::user_role_grants::delete_user_role_grant::*;

use super::PgUserRoleGrant;

#[async_trait]
impl<'a> Service<&'a DeleteUserRoleGrant> for Database {
    type Response = UserRoleGrant;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_user_role_grant_query", skip(self))]
    async fn call(
        &self,
        params: &'a DeleteUserRoleGrant,
    ) -> Result<Self::Response, Self::Error> {
        let row = sqlx::query_as::<_, PgUserRoleGrant>(include_str!(
            "./delete_user_role_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.role_id)
        .fetch_one(&self.pool)
        .await?;

        let user_role_grant = row.into();

        Ok(user_role_grant)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    

    #[ignore]
    #[sqlx::test]
    async fn it_should_be_able_to_delete_an_existing_user_role_grant(
        _pool: PgPool,
    ) {
    }
}
