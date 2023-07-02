use oxidauth_repository::refresh_tokens::{delete_refresh_token_by_expires_at::*, RefreshTokenRow};

use crate::prelude::*;

#[async_trait]
impl DeleteRefreshTokenByExpiresAt for Database {
    async fn delete_refresh_token_by_expires_at(
        &self,
        expires_at: DateTime<Utc>,
    ) -> Result<Vec<RefreshTokenRow>, DeleteRefreshTokenByExpiresAtError> {
        let result = sqlx::query_as::<_, super::RefreshTokenRow>(include_str!(
            "./delete_refresh_token_by_expires_at.sql"
        ))
        .bind(expires_at)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| DeleteRefreshTokenByExpiresAtError {})?
        .into_iter()
        .map(Into::into)
        .collect();

        Ok(result)
    }
}
