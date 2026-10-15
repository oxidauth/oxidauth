use oxidauth_repository::oauth_states::consume_oauth_state::*;

use super::*;

#[async_trait]
impl ConsumeOAuthStateQuery for PgOAuthStateRepository {
    #[tracing::instrument(name = "consume_oauth_state_query", skip(self))]
    async fn consume_oauth_state(&self, params: &ConsumeOAuthState) -> Result<bool, BoxedError> {
        // DELETE ... RETURNING + fetch_optional is the atomic single-use
        // consume: two concurrent requests for the same digest serialize on
        // the row lock, and exactly one observes the row (OXA-000010)
        let consumed = sqlx::query_scalar::<_, Vec<u8>>(include_str!("./consume_oauth_state.sql"))
            .bind(&params.state_hash)
            .bind(params.client_key)
            .fetch_optional(&self.db.write_pool())
            .await?;

        Ok(consumed.is_some())
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_repository::oauth_states::insert_oauth_state::{
        CreateOAuthState,
        InsertOAuthStateQuery,
    };
    use sha2::{Digest, Sha256};
    use sqlx::PgPool;

    use super::*;
    use crate::MIGRATOR;

    fn repo(pool: &PgPool) -> PgOAuthStateRepository {
        PgOAuthStateRepository::new(Database::from_pool(pool.clone()))
    }

    fn hash(state: &str) -> Vec<u8> {
        Sha256::digest(state.as_bytes()).to_vec()
    }

    async fn mint(pool: &PgPool, state: &str, client_key: uuid::Uuid) {
        repo(pool)
            .insert_oauth_state(&CreateOAuthState {
                state_hash: hash(state),
                client_key,
                expires_at: Utc::now() + chrono::Duration::minutes(10),
            })
            .await
            .expect("mint should succeed");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn it_should_consume_once_and_refuse_the_replay(pool: PgPool) {
        let client_key = Uuid::new_v4();
        mint(&pool, "single-use-state", client_key).await;

        let first = repo(&pool)
            .consume_oauth_state(&ConsumeOAuthState {
                state_hash: hash("single-use-state"),
                client_key,
            })
            .await
            .expect("consume should run");
        assert!(first, "the live nonce is consumed");

        let second = repo(&pool)
            .consume_oauth_state(&ConsumeOAuthState {
                state_hash: hash("single-use-state"),
                client_key,
            })
            .await
            .expect("consume should run");
        assert!(
            !second,
            "the spent nonce must not verify again — single-use"
        );
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn it_should_refuse_unknown_states_and_wrong_client_keys(pool: PgPool) {
        let client_key = Uuid::new_v4();
        mint(&pool, "bound-state", client_key).await;

        let unknown = repo(&pool)
            .consume_oauth_state(&ConsumeOAuthState {
                state_hash: hash("never-minted"),
                client_key,
            })
            .await
            .expect("consume should run");
        assert!(!unknown, "an unknown digest cannot verify");

        let wrong_key = repo(&pool)
            .consume_oauth_state(&ConsumeOAuthState {
                state_hash: hash("bound-state"),
                client_key: Uuid::new_v4(),
            })
            .await
            .expect("consume should run");
        assert!(
            !wrong_key,
            "a nonce minted for authority A must not verify against authority B"
        );

        // the refused consume did not spend it: the right key still works
        let right = repo(&pool)
            .consume_oauth_state(&ConsumeOAuthState {
                state_hash: hash("bound-state"),
                client_key,
            })
            .await
            .expect("consume should run");
        assert!(right, "a wrong-key miss must not spend the nonce");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn it_should_refuse_expired_states(pool: PgPool) {
        sqlx::query(
            "INSERT INTO oauth_states (state_hash, client_key, expires_at) VALUES ($1, $2, NOW() - interval '1 second')",
        )
        .bind(hash("stale-state"))
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await
        .expect("seed stale row");

        let stale = repo(&pool)
            .consume_oauth_state(&ConsumeOAuthState {
                state_hash: hash("stale-state"),
                client_key: Uuid::new_v4(),
            })
            .await
            .expect("consume should run");
        assert!(!stale, "an expired nonce cannot verify");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn it_should_consume_exactly_once_under_concurrent_identical_calls(pool: PgPool) {
        let client_key = Uuid::new_v4();
        mint(&pool, "raced-state", client_key).await;

        let params = ConsumeOAuthState {
            state_hash: hash("raced-state"),
            client_key,
        };
        let repo_a = repo(&pool);
        let repo_b = repo(&pool);

        let (a, b) = tokio::join!(
            repo_a.consume_oauth_state(&params),
            repo_b.consume_oauth_state(&params),
        );

        let a = a.expect("consume a should run");
        let b = b.expect("consume b should run");
        assert!(
            a ^ b,
            "exactly one concurrent replay must win, got a={a} b={b}"
        );
    }
}
