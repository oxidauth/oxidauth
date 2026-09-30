use oxidauth_repository::refresh_tokens::select_refresh_token_by_id::*;

use super::*;

#[async_trait]
impl SelectRefreshTokenByIdQuery for PgRefreshTokenRepository {
    #[tracing::instrument(name = "select_refresh_token_by_id_query", skip(self))]
    async fn select_refresh_token_by_id(
        &self,
        params: &FindRefreshTokenById,
    ) -> Result<RefreshToken, BoxedError> {
        let result =
            sqlx::query_as::<_, PgRefreshToken>(include_str!("./select_refresh_token_by_id.sql"))
                .bind(params.refresh_token_id)
                .fetch_one(&self.db.read_pool())
                .await?;

        let refresh_token = result.into();

        Ok(refresh_token)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::{seed_authority, seed_refresh_token, seed_user};

    fn repo(pool: &PgPool) -> PgRefreshTokenRepository {
        PgRefreshTokenRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_a_refresh_token_by_id(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "refresh-find").await;
        let expires_at = DateTime::from_timestamp(1_893_456_000, 0).expect("valid ts");
        let id = seed_refresh_token(&pool, user, authority, expires_at).await;

        let found = repo(&pool)
            .select_refresh_token_by_id(&FindRefreshTokenById {
                refresh_token_id: id,
            })
            .await
            .expect("query should succeed");

        assert_eq!(found.id, id);
        assert_eq!(found.user_id, user);
        assert_eq!(found.authority_id, authority);
        assert_eq!(found.expires_at, expires_at);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_token_is_missing(pool: PgPool) {
        let err = repo(&pool)
            .select_refresh_token_by_id(&FindRefreshTokenById {
                refresh_token_id: Uuid::new_v4(),
            })
            .await
            .err()
            .unwrap_or_else(|| panic!("an unknown id must not return Ok, got Ok"));
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }
}
