use oxidauth_repository::public_keys::select_all_public_keys::*;

use super::*;

#[async_trait]
impl SelectAllPublicKeysQuery for PgPublicKeyRepository {
    #[tracing::instrument(name = "select_all_public_keys_query", skip(self))]
    async fn select_all_public_keys(
        &self,
        params: &ListAllPublicKeys,
    ) -> Result<Vec<PublicKey>, BoxedError> {
        let public_key =
            sqlx::query_as::<_, PgPublicSanitizedKey>(include_str!("./select_all_public_keys.sql"))
                .fetch_all(&self.db.read_pool())
                .await?
                .into_iter()
                .map(|public_key| public_key.try_into())
                .collect::<Result<Vec<PublicKey>, BoxedError>>()?;

        Ok(public_key)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::seed_public_key;

    fn repo(pool: &PgPool) -> PgPublicKeyRepository {
        PgPublicKeyRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_no_keys_for_an_empty_table(pool: PgPool) {
        let keys = repo(&pool)
            .select_all_public_keys(&ListAllPublicKeys)
            .await
            .expect("query should succeed");
        assert!(keys.is_empty(), "expected no rows, got: {keys:?}");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_every_key_for_the_jwks_payload(pool: PgPool) {
        let first = seed_public_key(&pool, b"jwks-key-one").await;
        let second = seed_public_key(&pool, b"jwks-key-two").await;
        let third = seed_public_key(&pool, b"jwks-key-three").await;
        // Guard (OXA-000019 precedent): force the tie — seed rows tie on created_at
        // only if NOW() never ticks between INSERTs, and while updated_at equals
        // created_at an updated_at-first orderer is indistinguishable from the
        // correct one. Pin every row's created_at to the newest one, then drop the
        // max-id row's updated_at below that tie: `ORDER BY updated_at ASC, id ASC`
        // must then hoist the max-id row to the front — observably wrong — while
        // the correct orderer never reads updated_at and keeps it last.
        sqlx::query(
            "UPDATE public_keys SET created_at = (SELECT max(created_at) FROM public_keys), \
             updated_at = CASE WHEN id = (SELECT id FROM public_keys ORDER BY id DESC LIMIT 1) \
             THEN (SELECT max(created_at) FROM public_keys) - INTERVAL '1 hour' \
             ELSE (SELECT max(created_at) FROM public_keys) END",
        )
        .execute(&pool)
        .await
        .expect("diverge one row's updated_at");

        let repo = repo(&pool);
        let keys = repo
            .select_all_public_keys(&ListAllPublicKeys)
            .await
            .expect("query should succeed");

        assert_eq!(keys.len(), 3);

        assert!(
            keys.windows(2)
                .all(|w| (w[0].created_at, w[0].id) <= (w[1].created_at, w[1].id)),
            "rows must be returned in (created_at, id) order"
        );

        let again = repo
            .select_all_public_keys(&ListAllPublicKeys)
            .await
            .expect("repeated query should succeed");
        assert_eq!(
            keys.iter()
                .map(|k| k.id)
                .collect::<Vec<_>>(),
            again
                .iter()
                .map(|k| k.id)
                .collect::<Vec<_>>(),
            "repeated calls must return identical id sequences"
        );

        let mut ids: Vec<Uuid> = keys
            .iter()
            .map(|k| k.id)
            .collect();
        ids.sort();
        let mut expected = vec![first, second, third];
        expected.sort();
        assert_eq!(
            ids, expected,
            "every stored key must appear in the jwks source"
        );

        let mut payloads: Vec<&str> = keys
            .iter()
            .map(|k| k.public_key.as_str())
            .collect();
        payloads.sort_unstable();
        assert_eq!(
            payloads,
            vec!["jwks-key-one", "jwks-key-three", "jwks-key-two"],
            "public bytes must decode to UTF-8 strings"
        );
        assert!(
            keys.iter()
                .all(|k| !format!("{k:?}").contains("private-material")),
            "the sanitized projection must never carry private key material"
        );
    }
}
