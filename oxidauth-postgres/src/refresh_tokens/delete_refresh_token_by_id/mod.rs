use oxidauth_repository::refresh_tokens::delete_refresh_token_by_id::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a DeleteRefreshTokenById> for Database {
    type Response = RefreshToken;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_refresh_token_query_by_id", skip(self))]
    async fn call(&self, params: &'a DeleteRefreshTokenById) -> Result<RefreshToken, BoxedError> {
        let result =
            sqlx::query_as::<_, PgRefreshToken>(include_str!("./delete_refresh_token_by_id.sql"))
                .bind(params.refresh_token_id)
                .fetch_one(&self.pool)
                .await?;

        let refresh_token = result.into();

        Ok(refresh_token)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[ignore]
    #[sqlx::test]
    async fn it_should_delete_a_refresh_token_by_id_successfully(_pool: PgPool) {}
}
