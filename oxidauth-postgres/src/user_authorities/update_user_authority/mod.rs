use crate::Database;

use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::{
        update_user_authority::UpdateUserAuthority, UserAuthority,
    },
};
use oxidauth_repository::users::insert_user::*;

use super::PgUserAuthority;

#[async_trait]
impl<'a> Service<&'a UpdateUserAuthority> for Database {
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_user_authority_query", skip(self))]
    async fn call(
        &self,
        params: &'a UpdateUserAuthority,
    ) -> Result<Self::Response, Self::Error> {
        let row = sqlx::query_as::<_, PgUserAuthority>(include_str!(
            "./update_user_authority.sql"
        ))
        .bind(params.user_id)
        .bind(params.authority_id)
        .bind(&*params.params)
        .fetch_one(&self.pool)
        .await?;

        let user_authority = row.into();

        Ok(user_authority)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_be_able_to_update_an_existing_user_authority(
        pool: PgPool,
    ) {
    }
}
