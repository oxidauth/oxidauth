use oxidauth_repository::totp_secrets::{
    insert_totp_secret::InsertTotpSecretParams,
    insert_totp_secrets::*,
};

use super::*;
use crate::totp_secrets::insert_totp_secret::insert_totp_secret_query;

#[async_trait]
impl InsertTotpSecretsQuery for PgTotpSecretRepository {
    #[tracing::instrument(name = "insert_totp_secrets_query", skip(self, params))]
    async fn insert_totp_secrets(
        &self,
        params: &InsertTotpSecretsParams,
    ) -> Result<(), BoxedError> {
        let mut tx = self
            .db
            .write_pool()
            .begin()
            .await?;

        for (user_id, secret_key) in params
            .user_id_and_secrets
            .clone()
            .into_iter()
        {
            insert_totp_secret_query(
                &mut tx,
                &InsertTotpSecretParams {
                    user_id,
                    secret_key,
                },
            )
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{Database, test_fixtures::seed_user};

    fn repo(pool: &PgPool) -> PgTotpSecretRepository {
        PgTotpSecretRepository::new(Database::from_pool(pool.clone()))
    }

    async fn secret_count(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM totp_secrets")
            .fetch_one(pool)
            .await
            .expect("count should run")
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_batch_insert_secrets(pool: PgPool) {
        let user_a = seed_user(&pool).await;
        let user_b = seed_user(&pool).await;

        repo(&pool)
            .insert_totp_secrets(&InsertTotpSecretsParams {
                user_id_and_secrets: vec![
                    (user_a, "SECRETAAAA".to_owned()),
                    (user_b, "SECRETBBBB".to_owned()),
                ],
            })
            .await
            .expect("batch insert should succeed");

        let rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT user_id, totp_secret FROM totp_secrets ORDER BY totp_secret")
                .fetch_all(&pool)
                .await
                .expect("rows should be readable");
        assert_eq!(
            rows,
            vec![
                (user_a, "SECRETAAAA".to_owned()),
                (user_b, "SECRETBBBB".to_owned()),
            ]
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_roll_back_the_whole_batch_when_one_insert_fails(pool: PgPool) {
        // 2FA authority rollout: one FK violation in the batch must not leave
        // partially-enrolled users behind
        let user = seed_user(&pool).await;
        let unknown_user = Uuid::new_v4();

        let err = repo(&pool)
            .insert_totp_secrets(&InsertTotpSecretsParams {
                user_id_and_secrets: vec![
                    (user, "SECRETVALID".to_owned()),
                    (unknown_user, "SECRETINVALID".to_owned()),
                ],
            })
            .await
            .err()
            .unwrap_or_else(|| panic!("the FK violation must abort the batch, got Ok"));
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        match sqlx_err {
            sqlx::Error::Database(db_err) => {
                assert_eq!(
                    db_err.code().as_deref(),
                    Some("23503"),
                    "expected the users FK violation, got: {db_err}"
                )
            },
            other => panic!("expected a database error, got: {other:?}"),
        }

        assert_eq!(
            secret_count(&pool).await,
            0,
            "the explicit transaction must roll the valid row back too"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_accept_an_empty_batch(pool: PgPool) {
        repo(&pool)
            .insert_totp_secrets(&InsertTotpSecretsParams {
                user_id_and_secrets: vec![],
            })
            .await
            .expect("an empty batch is a no-op");
        assert_eq!(secret_count(&pool).await, 0);
    }
}
