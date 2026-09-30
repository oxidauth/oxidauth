use oxidauth_repository::public_keys::insert_public_key::*;

use super::*;

#[async_trait]
impl InsertPublicKeyQuery for PgPublicKeyRepository {
    #[tracing::instrument(name = "insert_public_key_query", skip(self, params))]
    async fn insert_public_key(
        &self,
        params: &InsertPublicKeyParams,
    ) -> Result<PublicKey, BoxedError> {
        let result = sqlx::query_as::<_, PgPublicKey>(include_str!("./insert_public_key.sql"))
            .bind(params.id)
            .bind(&params.private_key)
            .bind(&params.public_key)
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

    fn repo(pool: &PgPool) -> PgPublicKeyRepository {
        PgPublicKeyRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_insert_with_explicit_id_and_round_trip_bytes(pool: PgPool) {
        let id = Uuid::new_v4();
        let public_key = b"-----BEGIN PUBLIC KEY-----PEM-----END PUBLIC KEY-----".to_vec();
        let private_key = b"-----BEGIN PRIVATE KEY-----PEM-----END PRIVATE KEY-----".to_vec();

        let stored = repo(&pool)
            .insert_public_key(&InsertPublicKeyParams {
                id: Some(id),
                public_key: public_key.clone(),
                private_key: private_key.clone(),
            })
            .await
            .expect("insert should succeed");

        assert_eq!(stored.id, id, "an explicit id is honoured");
        assert_eq!(
            stored.public_key,
            String::from_utf8(public_key.clone()).expect("valid utf8"),
            "the response decodes the public bytes as UTF-8"
        );
        assert!(stored.created_at.timestamp() > 0);
        assert!(stored.updated_at >= stored.created_at);

        let (pub_bytes, priv_bytes): (Vec<u8>, Vec<u8>) =
            sqlx::query_as("SELECT public_key, private_key FROM public_keys WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .expect("row must be persisted");
        assert_eq!(pub_bytes, public_key);
        assert_eq!(
            priv_bytes, private_key,
            "the private half is stored alongside"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_generate_an_id_when_none_is_given(pool: PgPool) {
        let stored = repo(&pool)
            .insert_public_key(&InsertPublicKeyParams {
                id: None,
                public_key: b"pub-key-material".to_vec(),
                private_key: b"priv-key-material".to_vec(),
            })
            .await
            .expect("insert should succeed");

        assert!(
            !stored.id.is_nil(),
            "COALESCE($1, uuid_generate_v4()) fills the id"
        );
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public_keys WHERE id = $1")
            .bind(stored.id)
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_accept_identical_key_material_twice(pool: PgPool) {
        // BUG(pinned): no uniqueness anywhere in public_keys — re-inserting identical
        // key material persists a second row (duplicates are not detected).
        let params = InsertPublicKeyParams {
            id: None,
            public_key: b"same-pub".to_vec(),
            private_key: b"same-priv".to_vec(),
        };
        let repo = repo(&pool);

        repo.insert_public_key(&params)
            .await
            .expect("first insert");
        repo.insert_public_key(&params)
            .await
            .expect("duplicate insert is accepted");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public_keys")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 2);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_persist_the_row_but_error_when_public_bytes_are_not_utf8(pool: PgPool) {
        // BUG(pinned): the write succeeds and only THEN the response conversion does
        // String::from_utf8 — the caller sees Err while the row is already committed
        // to the (test) transaction, so a naive retry silently accumulates rows.
        let err = repo(&pool)
            .insert_public_key(&InsertPublicKeyParams {
                id: None,
                public_key: vec![0xFF, 0xFE, 0x00, 0x01],
                private_key: b"priv".to_vec(),
            })
            .await
            .err()
            .unwrap_or_else(|| panic!("invalid UTF-8 must surface an error, got Ok"));
        assert!(
            err.downcast_ref::<std::string::FromUtf8Error>()
                .is_some(),
            "expected a FromUtf8Error, got: {err:?}"
        );

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public_keys")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1, "the row is persisted despite the Err");
    }
}
