use oxidauth_repository::refresh_tokens::{delete_refresh_token_by_expires_at::*, RefreshTokenRow};

use crate::prelude::*;

#[async_trait]
impl DeleteRefreshTokenByExpiresAt for Database {
    async fn delete_refresh_token_by_expires_at(
        &self,
        refresh_token_id: Uuid,
    ) -> Result<RefreshTokenRow, DeleteRefreshTokenByExpiresAtError> {
        let result = sqlx::query_as::<_, super::RefreshTokenRow>(include_str!(
            "./delete_refresh_token_by_expires_at.sql"
        ))
        .bind(refresh_token_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeleteRefreshTokenByExpiresAtError {})?;

        Ok(result)
    }
}
