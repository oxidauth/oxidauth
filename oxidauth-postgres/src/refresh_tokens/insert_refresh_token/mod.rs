use oxidauth_repository::refresh_tokens::insert_refresh_token::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a CreateRefreshToken> for Database {
    type Response = RefreshToken;
    type Error = BoxedError;

    #[tracing::instrument(name = "insert_refresh_token_query", skip(self))]
    async fn call(
        &self,
        params: &'a CreateRefreshToken,
    ) -> Result<RefreshToken, BoxedError> {
        let result = sqlx::query_as::<_, PgRefreshToken>(include_str!(
            "./insert_refresh_token.sql"
        ))
        .bind(None::<Uuid>)
        .bind(params.user_id)
        .bind(params.authority_id)
        .bind(params.expires_at)
        .fetch_one(&self.pool)
        .await?;

        let refresh_token = result.into();

        Ok(refresh_token)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_insert_a_refresh_token_successfully(pool: PgPool) {}
}
