use oxidauth_repository::refresh_tokens::select_refresh_token_by_id::*;

use crate::prelude::*;

use super::*;

#[async_trait]
impl<'a> Service<&'a FindRefreshTokenById> for Database {
    type Response = RefreshToken;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "select_refresh_token_by_id_query",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a FindRefreshTokenById,
    ) -> Result<RefreshToken, BoxedError> {
        let result = sqlx::query_as::<_, PgRefreshToken>(include_str!(
            "./select_refresh_token_by_id.sql"
        ))
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

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_select_a_refresh_token_by_id_successfully(pool: PgPool) {
    }
}
