use crate::Database;

use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::{
        create_user_authority::CreateUserAuthority, UserAuthority,
    },
};
use oxidauth_repository::users::insert_user::*;

use super::PgUserAuthority;

#[async_trait]
impl<'a> Service<&'a CreateUserAuthority> for Database {
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_user_authority_query", skip(self))]
    async fn call(
        &self,
        params: &'a CreateUserAuthority,
    ) -> Result<Self::Response, Self::Error> {
        let row = sqlx::query_as::<_, PgUserAuthority>(include_str!(
            "./insert_user_authority.sql"
        ))
        .bind(params.user_id)
        .bind(params.authority_id)
        .bind(&params.user_identifier)
        .bind(&params.params)
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
    async fn it_should_be_able_to_insert_a_new_user_authority(pool: PgPool) {}
}
