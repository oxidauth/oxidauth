use std::fmt;

use crate::Database;

use oxidauth_kernel::{error::BoxedError, user_authorities::UserAuthority};
use oxidauth_repository::{
    user_authorities::insert_user_authority::{
        InsertUserAuthority, InsertUserAuthorityQuery,
    },
    users::insert_user::*,
};

use super::PgUserAuthority;

#[async_trait]
impl InsertUserAuthorityQuery for Database {
    #[tracing::instrument(name = "insert_user_authority_query", skip(self))]
    async fn call(
        &self,
        params: impl Into<InsertUserAuthority> + Send + fmt::Debug + 'async_trait,
    ) -> Result<UserAuthority, BoxedError> {
        let params = params.into();

        let row = sqlx::query_as::<_, PgUserAuthority>(include_str!(
            "./insert_user_authority.sql"
        ))
        .bind(params.user_id)
        .bind(params.authority_id)
        .bind(&params.user_identifier)
        .bind(params.params.inner_value())
        .fetch_one(&self.pool)
        .await?;

        let user_authority = row.into();

        Ok(user_authority)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_be_able_to_insert_a_new_user_authority(_pool: PgPool) {}
}
