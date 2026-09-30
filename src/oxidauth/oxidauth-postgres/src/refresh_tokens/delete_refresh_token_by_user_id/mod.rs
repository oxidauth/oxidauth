use oxidauth_repository::refresh_tokens::delete_refresh_token_by_user_id::*;

use super::*;

/// Deletes every refresh token of `user_id`. Returns the deleted rows
/// (possibly empty); zero matches is a success with an empty Vec, never an
/// error.
#[async_trait]
impl DeleteRefreshTokenByUserIdQuery for PgRefreshTokenRepository {
    #[tracing::instrument(name = "delete_refresh_token_by_user_id_query", skip(self))]
    async fn delete_refresh_token_by_user_id(
        &self,
        params: &DeleteRefreshTokenByUserId,
    ) -> Result<Vec<RefreshToken>, BoxedError> {
        let result = sqlx::query_as::<_, PgRefreshToken>(include_str!(
            "./delete_refresh_token_by_user_id.sql"
        ))
        .bind(params.user_id)
        .fetch_all(&self.db.write_pool())
        .await?;

        let deleted: Vec<RefreshToken> = result
            .into_iter()
            .map(RefreshToken::from)
            .collect();

        tracing::info!(count = deleted.len(), "bulk refresh token revocation");

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::{seed_authority, seed_refresh_token_now, seed_user};

    fn repo(pool: &PgPool) -> PgRefreshTokenRepository {
        PgRefreshTokenRepository::new(Database::from_pool(pool.clone()))
    }

    async fn token_count(pool: &PgPool, user: Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens WHERE user_id = $1")
            .bind(user)
            .fetch_one(pool)
            .await
            .expect("count should run")
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_only_the_target_users_tokens(pool: PgPool) {
        let alice = seed_user(&pool).await;
        let bob = seed_user(&pool).await;
        let authority = seed_authority(&pool, "refresh-bulk").await;
        let alice_first = seed_refresh_token_now(&pool, alice, authority).await;
        let alice_second = seed_refresh_token_now(&pool, alice, authority).await;
        let bobs_token = seed_refresh_token_now(&pool, bob, authority).await;

        let deleted = repo(&pool)
            .delete_refresh_token_by_user_id(&DeleteRefreshTokenByUserId { user_id: alice })
            .await
            .expect("bulk delete should succeed");

        assert_eq!(deleted.len(), 2, "both of alice's tokens are reported");
        assert!(
            deleted
                .iter()
                .all(|t| t.user_id == alice),
            "every reported row must belong to alice"
        );
        // the DELETE has no ORDER BY on its RETURNING — compare id sets
        let mut deleted_ids: Vec<Uuid> = deleted
            .iter()
            .map(|t| t.id)
            .collect();
        deleted_ids.sort();
        let mut alice_ids = [alice_first, alice_second];
        alice_ids.sort();
        assert_eq!(
            deleted_ids, alice_ids,
            "the reported ids are exactly alice's two tokens"
        );

        assert_eq!(
            token_count(&pool, alice).await,
            0,
            "all of alice's tokens are gone"
        );
        assert_eq!(
            token_count(&pool, bob).await,
            1,
            "bob's tokens are out of scope"
        );

        let still_present: Uuid =
            sqlx::query_scalar("SELECT id FROM refresh_tokens WHERE user_id = $1")
                .bind(bob)
                .fetch_one(&pool)
                .await
                .expect("bob's token must survive");
        assert_eq!(still_present, bobs_token);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_an_empty_vec_when_the_user_has_no_tokens(pool: PgPool) {
        let alice = seed_user(&pool).await;
        let bob = seed_user(&pool).await;
        let authority = seed_authority(&pool, "refresh-bulk-empty").await;
        seed_refresh_token_now(&pool, bob, authority).await;

        let deleted = repo(&pool)
            .delete_refresh_token_by_user_id(&DeleteRefreshTokenByUserId { user_id: alice })
            .await
            .expect("empty bulk delete must succeed");
        assert!(
            deleted.is_empty(),
            "zero matches must be a success with an empty Vec"
        );

        assert_eq!(
            token_count(&pool, bob).await,
            1,
            "the other user's token stays"
        );
    }
}
