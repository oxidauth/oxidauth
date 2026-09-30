use oxidauth_repository::public_keys::select_public_key_by_id::*;

use super::*;

#[async_trait]
impl SelectPublicKeyByIdQuery for PgPublicKeyRepository {
    #[tracing::instrument(name = "select_public_key_by_id_query", skip(self))]
    async fn select_public_key_by_id(
        &self,
        public_key_id: &FindPublicKeyById,
    ) -> Result<PublicKey, BoxedError> {
        let public_key = sqlx::query_as::<_, PgPublicSanitizedKey>(include_str!(
            "./select_public_key_by_id.sql"
        ))
        .bind(public_key_id.public_key_id)
        .fetch_one(&self.db.read_pool())
        .await?
        .try_into()?;

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
    async fn it_should_find_the_sanitized_key_by_primary_id(pool: PgPool) {
        let id = seed_public_key(&pool, b"looked-up-public-key").await;
        let _other = seed_public_key(&pool, b"other-public-key").await;

        let found = repo(&pool)
            .select_public_key_by_id(&FindPublicKeyById { public_key_id: id })
            .await
            .expect("query should succeed");

        assert_eq!(found.id, id);
        assert_eq!(found.public_key, "looked-up-public-key");
        // the PgPublicSanitizedKey row never selects the private_key column, so the
        // jwks-facing type cannot carry it
        let debug = format!("{found:?}");
        assert!(
            !debug.contains("private-material"),
            "the response type must not expose private key material: {debug}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_key_is_missing(pool: PgPool) {
        let err = repo(&pool)
            .select_public_key_by_id(&FindPublicKeyById {
                public_key_id: Uuid::new_v4(),
            })
            .await
            .expect_err("an unknown id must not return Ok");
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }
}
