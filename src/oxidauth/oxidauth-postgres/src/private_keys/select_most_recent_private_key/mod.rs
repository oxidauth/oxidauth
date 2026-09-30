use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    private_keys::find_most_recent_private_key::FindMostRecentPrivateKey,
};
use oxidauth_repository::private_keys::select_most_recent_private_key::*;

use super::*;

#[async_trait]
impl SelectMostRecentPrivateKeyQuery for PgPrivateKeyRepository {
    #[tracing::instrument(name = "select_most_recent_private_key_query", skip(self))]
    async fn select_most_recent_private_key(
        &self,
        params: &FindMostRecentPrivateKey,
    ) -> Result<PrivateKey, BoxedError> {
        let result =
            sqlx::query_as::<_, PgPrivateKey>(include_str!("./select_most_recent_private_key.sql"))
                .fetch_one(&self.db.read_pool())
                .await?;

        let private_key = result.into();

        Ok(private_key)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;

    fn repo(pool: &PgPool) -> PgPrivateKeyRepository {
        PgPrivateKeyRepository::new(Database::from_pool(pool.clone()))
    }

    fn ts(secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs, 0).expect("valid ts")
    }

    async fn seed_key(
        pool: &PgPool,
        private_key: &[u8],
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO public_keys (id, public_key, private_key, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(b"pub".to_vec())
        .bind(private_key.to_vec())
        .bind(created_at)
        .bind(updated_at)
        .execute(pool)
        .await
        .expect("seed key");
        id
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_no_keys_exist(pool: PgPool) {
        let err = repo(&pool)
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await
            .expect_err("an empty table must yield RowNotFound");
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_pick_the_newest_created_at_regardless_of_insert_order(pool: PgPool) {
        let old = seed_key(&pool, b"key-old", ts(1_000), ts(1_000)).await;
        let newest = seed_key(&pool, b"key-new", ts(3_000), ts(3_000)).await;
        // mid is inserted last AND has the newest updated_at: if the ordering column
        // were updated_at (or physical order), this row would win — it must not
        let mid = seed_key(&pool, b"key-mid", ts(2_000), ts(9_999)).await;

        let winner = repo(&pool)
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await
            .expect("query should succeed");

        assert_eq!(
            winner.id, newest,
            "ORDER BY created_at DESC must pick key {newest}"
        );
        assert_ne!(winner.id, old);
        assert_ne!(winner.id, mid, "updated_at must not drive the ordering");
        assert_eq!(
            winner.private_key,
            b"key-new".to_vec(),
            "the payload of the newest key must come back intact"
        );
        assert_eq!(winner.created_at, ts(3_000));
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_deterministically_break_same_created_at_ties(pool: PgPool) {
        // `created_at` carries no uniqueness constraint, so the ORDER BY has to be a
        // total order by itself: ties break to the greater `id` — never to `updated_at`,
        // never to whatever row order the plan happened to produce.
        let same = ts(5_000);
        let first = seed_key(&pool, b"key-first", same, ts(5_001)).await;
        let second = seed_key(&pool, b"key-second", same, ts(5_002)).await;

        let winner = repo(&pool)
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await
            .expect("the tie must still return exactly one row");

        assert_eq!(
            winner.id,
            std::cmp::max(first, second),
            "ties must break to the greater id"
        );
        assert_ne!(winner.id, std::cmp::min(first, second));
        assert_eq!(winner.created_at, same);
    }

    // The write side of the same defect: NOW() is the *transaction* start time, so a
    // rotation batch that inserts several keys inside one explicit transaction (the
    // `insert_totp_secrets` pattern) stamps every row with an identical `created_at`,
    // leaving zero ordering information in it.
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_pick_the_last_key_of_a_same_transaction_batch(pool: PgPool) {
        // The ids are picked so the FIRST insert owns the greater id: if the batch tied on
        // `created_at`, the `id DESC` tie-break would deterministically keep the superseded
        // row, so only per-statement timestamps can make the last-inserted key win.
        let first_id = Uuid::from_u128(u128::MAX);
        let second_id = Uuid::from_u128(1);

        let mut tx = pool
            .begin()
            .await
            .expect("open the batch");
        insert_key_in_tx(&mut tx, first_id, b"key-batch-first").await;
        insert_key_in_tx(&mut tx, second_id, b"key-batch-second").await;
        tx.commit()
            .await
            .expect("commit the batch");

        let winner = repo(&pool)
            .select_most_recent_private_key(&FindMostRecentPrivateKey {})
            .await
            .expect("the batch must yield exactly one key");

        assert_eq!(
            winner.id, second_id,
            "the key inserted last in the batch must be the signing key"
        );
        assert_eq!(
            winner.private_key,
            b"key-batch-second".to_vec(),
            "the payload of the last-inserted key must come back intact"
        );
    }

    /// Runs the real `insert_public_key.sql` on an open transaction, the way a bulk
    /// rotation would: `insert_totp_secrets` wraps its INSERTs in `write_pool().begin()`.
    async fn insert_key_in_tx(conn: &mut sqlx::PgConnection, id: Uuid, private_key: &[u8]) {
        sqlx::query(include_str!(
            "../../public_keys/insert_public_key/insert_public_key.sql"
        ))
        .bind(id)
        .bind(private_key.to_vec())
        .bind(b"pub".to_vec())
        .execute(conn)
        .await
        .expect("insert_public_key.sql must run inside the batch");
    }
}
