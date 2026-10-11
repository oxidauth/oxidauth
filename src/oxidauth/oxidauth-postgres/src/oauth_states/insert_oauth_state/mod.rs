use oxidauth_repository::oauth_states::insert_oauth_state::*;

use super::*;

#[async_trait]
impl InsertOAuthStateQuery for PgOAuthStateRepository {
    #[tracing::instrument(name = "insert_oauth_state_query", skip(self))]
    async fn insert_oauth_state(&self, params: &CreateOAuthState) -> Result<(), BoxedError> {
        // the statement also garbage-collects expired nonces (OXA-000010):
        // no cron, the flow's own churn keeps the table bounded
        sqlx::query(include_str!("./insert_oauth_state.sql"))
            .bind(&params.state_hash)
            .bind(params.client_key)
            .bind(params.expires_at)
            .execute(&self.db.write_pool())
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

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

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn it_should_store_the_digest_never_the_raw_state(pool: PgPool) {
        let state = "nonce-value-abc";
        let client_key = Uuid::new_v4();

        repo(&pool)
            .insert_oauth_state(&CreateOAuthState {
                state_hash: hash(state),
                client_key,
                expires_at: Utc::now() + chrono::Duration::minutes(10),
            })
            .await
            .expect("insert should succeed");

        let stored: Vec<u8> =
            sqlx::query_scalar("SELECT state_hash FROM oauth_states WHERE client_key = $1")
                .bind(client_key)
                .fetch_one(&pool)
                .await
                .expect("stored digest");
        assert_eq!(stored, hash(state), "the stored key is sha256(state)");

        let raw_present: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM oauth_states WHERE encode(state_hash, 'escape') = $1",
        )
        .bind(state)
        .fetch_one(&pool)
        .await
        .expect("count should run");
        assert_eq!(raw_present, 0, "the raw state string must never be stored");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn it_should_garbage_collect_expired_rows_on_insert(pool: PgPool) {
        // seed an expired row directly (the impl would GC it on the next mint)
        sqlx::query("INSERT INTO oauth_states (state_hash, client_key, expires_at) VALUES ($1, $2, NOW() - interval '1 minute')")
            .bind(hash("expired-state"))
            .bind(Uuid::new_v4())
            .execute(&pool)
            .await
            .expect("seed expired row");

        repo(&pool)
            .insert_oauth_state(&CreateOAuthState {
                state_hash: hash("fresh-state"),
                client_key: Uuid::new_v4(),
                expires_at: Utc::now() + chrono::Duration::minutes(10),
            })
            .await
            .expect("insert should succeed");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM oauth_states")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1, "the expired row is collected by the same call");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn it_should_reject_a_duplicate_digest(pool: PgPool) {
        let params = CreateOAuthState {
            state_hash: hash("clashing-state"),
            client_key: Uuid::new_v4(),
            expires_at: Utc::now() + chrono::Duration::minutes(10),
        };

        repo(&pool)
            .insert_oauth_state(&params)
            .await
            .expect("first insert should succeed");

        let err = repo(&pool)
            .insert_oauth_state(&params)
            .await
            .err()
            .unwrap_or_else(|| panic!("a duplicate digest must violate the PK, got Ok"));
        crate::test_fixtures::assert_sql_state(err, "23505");
    }
}
