use oxidauth_kernel::user_authorities::{
    list_user_authorities_by_user_id::ListUserAuthoritiesByUserId,
    UserAuthorityWithAuthority,
};

use crate::prelude::*;

use super::PgUserAuthorityWithAuthority;

#[async_trait]
impl<'a> Service<&'a ListUserAuthoritiesByUserId> for Database {
    type Response = Vec<UserAuthorityWithAuthority>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_user_authorities_by_user_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a ListUserAuthoritiesByUserId,
    ) -> Result<Self::Response, Self::Error> {
        let user_authorities =
            sqlx::query_as::<_, PgUserAuthorityWithAuthority>(include_str!(
                "./select_user_authorities_by_user_id.sql"
            ))
            .bind(params.user_id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|u| u.try_into())
            .collect::<Result<Vec<UserAuthorityWithAuthority>, BoxedError>>()?;

        Ok(user_authorities)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_query_user_authorities_by_user_id_successfully(
        _pool: PgPool,
    ) {
    }
}
