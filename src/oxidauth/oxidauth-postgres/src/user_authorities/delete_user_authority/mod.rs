use oxidauth_kernel::user_authorities::{
    delete_user_authority::DeleteUserAuthority, UserAuthority,
};
use oxidauth_repository::user_authorities::delete_user_authority::*;

use crate::prelude::*;

use super::PgUserAuthority;

#[async_trait]
impl<'a> Service<&'a DeleteUserAuthority> for Database {
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_user_authority_query", skip(self))]
    async fn call(
        &self,
        params: &'a DeleteUserAuthority,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgUserAuthority>(include_str!(
            "./delete_user_authority.sql"
        ))
        .bind(params.user_id)
        .bind(params.authority_id)
        .fetch_one(&self.pool)
        .await?;

        let user_authority = result.into();

        Ok(user_authority)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_delete_a_user_authority_successfully(_pool: PgPool) {}
}
