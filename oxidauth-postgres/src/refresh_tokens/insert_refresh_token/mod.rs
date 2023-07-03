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
                .bind(&params.id)
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

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_insert_a_refresh_token_successfully(pool: PgPool) {
        let db = Database { pool };

        let refresh_token_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let authority_id = Uuid::new_v4();

        // @GEORGE - not sure abou the expires at
        let insert_params = InsertRefreshTokenParams {
            id: Some(refresh_token_id),
            user_id: user_id,
            authority_id: authority_id,
            expires_at: Utc::now(),
        };

        match db.insert_refresh_token(&insert_params).await {
            Ok(refresh_token) => {
                assert_eq!(refresh_token_id, refresh_token.id);
                assert_eq!(user_id, refresh_token.user_id);
                assert_eq!(authority_id, refresh_token.authority_id);
                assert_eq!(insert_params.expires_at, refresh_token.expires_at);
            }
            Err(_) => unreachable!(),
        }
    }
}
