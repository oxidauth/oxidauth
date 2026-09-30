use oxidauth_kernel::public_keys::delete_public_key::DeletePublicKey;
use oxidauth_repository::public_keys::delete_public_key::*;

use super::*;

#[async_trait]
impl DeletePublicKeyQuery for PgPublicKeyRepository {
    #[tracing::instrument(name = "delete_public_key_query", skip(self))]
    async fn delete_public_key(&self, params: &DeletePublicKey) -> Result<PublicKey, BoxedError> {
        let result = sqlx::query_as::<_, PgPublicKey>(include_str!("./delete_public_key.sql"))
            .bind(params.public_key_id)
            .fetch_one(&self.db.write_pool())
            .await?;

        let public_key = result.try_into()?;

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
    async fn it_should_delete_and_return_the_removed_key(pool: PgPool) {
        let id = seed_public_key(&pool, b"doomed-public-key").await;

        let deleted = repo(&pool)
            .delete_public_key(&DeletePublicKey { public_key_id: id })
            .await
            .expect("delete should succeed");

        assert_eq!(deleted.id, id);
        assert_eq!(deleted.public_key, "doomed-public-key");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public_keys")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_key_is_missing(pool: PgPool) {
        let err = repo(&pool)
            .delete_public_key(&DeletePublicKey {
                public_key_id: Uuid::new_v4(),
            })
            .await
            .expect_err("DELETE ... RETURNING with no matching row yields RowNotFound");
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }
}
