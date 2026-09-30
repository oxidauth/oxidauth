use oxidauth_repository::refresh_tokens::insert_refresh_token::*;

use super::*;

#[async_trait]
impl InsertRefreshTokenQuery for PgRefreshTokenRepository {
    #[tracing::instrument(name = "insert_refresh_token_query", skip(self))]
    async fn insert_refresh_token(
        &self,
        params: &CreateRefreshToken,
    ) -> Result<RefreshToken, BoxedError> {
        let result =
            sqlx::query_as::<_, PgRefreshToken>(include_str!("./insert_refresh_token.sql"))
                .bind(None::<Uuid>)
                .bind(params.user_id)
                .bind(params.authority_id)
                .bind(params.expires_at)
                .fetch_one(&self.db.write_pool())
                .await?;

        let refresh_token = result.into();

        Ok(refresh_token)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::{assert_sql_state, seed_authority, seed_user};

    fn repo(pool: &PgPool) -> PgRefreshTokenRepository {
        PgRefreshTokenRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_round_trip_expires_at(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "refresh-insert").await;
        // the service compares expires_at against epoch seconds, so use an exact
        // whole-second timestamp: Postgres timestamptz keeps microseconds, chrono ns
        let expires_at = DateTime::from_timestamp(1_893_456_000, 0).expect("2030-01-01T00:00:00Z");

        let token = repo(&pool)
            .insert_refresh_token(&CreateRefreshToken {
                user_id: user,
                authority_id: authority,
                expires_at,
            })
            .await
            .expect("insert should succeed");

        assert_eq!(token.user_id, user);
        assert_eq!(token.authority_id, authority);
        assert_eq!(
            token.expires_at, expires_at,
            "expires_at must survive the DB round-trip untouched"
        );
        assert!(
            !token.id.is_nil(),
            "the id is always generated: the impl binds None regardless of params"
        );
        assert!(token.created_at.timestamp() > 0);
        assert!(token.updated_at >= token.created_at);

        let stored: DateTime<Utc> =
            sqlx::query_scalar("SELECT expires_at FROM refresh_tokens WHERE id = $1")
                .bind(token.id)
                .fetch_one(&pool)
                .await
                .expect("stored expires_at");
        assert_eq!(stored, expires_at);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_unknown_user_or_authority_ids(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "refresh-fk").await;
        let expires_at = DateTime::from_timestamp(1_893_456_000, 0).expect("valid ts");

        let err = repo(&pool)
            .insert_refresh_token(&CreateRefreshToken {
                user_id: Uuid::new_v4(),
                authority_id: authority,
                expires_at,
            })
            .await
            .err()
            .unwrap_or_else(|| panic!("an unknown user id must violate the users FK, got Ok"));
        assert_sql_state(err, "23503");

        let err = repo(&pool)
            .insert_refresh_token(&CreateRefreshToken {
                user_id: user,
                authority_id: Uuid::new_v4(),
                expires_at,
            })
            .await
            .err()
            .unwrap_or_else(|| {
                panic!("an unknown authority id must violate the authorities FK, got Ok")
            });
        assert_sql_state(err, "23503");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0);
    }
}
