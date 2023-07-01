use oxidauth_repository::refresh_tokens::{insert_refresh_token::*, RefreshTokenRow};

use crate::prelude::*;

#[async_trait]
impl InsertRefreshToken for Database {
    async fn insert_refresh_token(
        &self,
        params: &InsertRefreshTokenParams,
    ) -> Result<RefreshTokenRow, InsertRefreshTokenError> {
        let result =
            sqlx::query_as::<_, super::RefreshTokenRow>(include_str!("./insert_refresh_token.sql"))
                .bind(&params.user_id)
                .bind(&params.authority_id)
                .bind(&params.expires_at)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| InsertRefreshTokenError {})?;

        Ok(result)
    }
}
