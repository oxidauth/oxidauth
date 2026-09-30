use oxidauth_kernel::totp_secrets::{
    TOTPSecret,
    find_totp_secret_by_user_id::FindTOTPSecretByUserId,
};
use oxidauth_repository::totp_secrets::select_totp_secret_by_user_id::*;
use sqlx::PgConnection;

use super::*;

#[async_trait]
impl SelectTOTPSecrețByUserIdQuery for PgTotpSecretRepository {
    #[tracing::instrument(name = "select_totp_secret_by_user_id_query", skip(self))]
    async fn select_totp_secret_by_user_id(
        &self,
        params: &FindTOTPSecretByUserId,
    ) -> Result<TOTPSecret, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let result = select_totp_secret_by_user_id_query(&mut conn, params.user_id).await?;

        let secret = TOTPSecret {
            secret: result.totp_secret,
        };

        Ok(secret)
    }
}

pub async fn select_totp_secret_by_user_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<PgTotpSecret, BoxedError> {
    let result =
        sqlx::query_as::<_, PgTotpSecret>(include_str!("./select_totp_secret_by_user_id.sql"))
            .bind(user_id)
            .fetch_one(conn)
            .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_totp_secret, seed_user},
    };

    fn repo(pool: &PgPool) -> PgTotpSecretRepository {
        PgTotpSecretRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_the_secret_of_a_user(pool: PgPool) {
        let user = seed_user(&pool).await;
        seed_totp_secret(&pool, user, "JBSWY3DPEHPK3PXP").await;

        let found = repo(&pool)
            .select_totp_secret_by_user_id(&FindTOTPSecretByUserId { user_id: user })
            .await
            .expect("query should succeed");
        assert_eq!(found.secret, "JBSWY3DPEHPK3PXP");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_user_has_no_secret(pool: PgPool) {
        let user = seed_user(&pool).await;

        let err = repo(&pool)
            .select_totp_secret_by_user_id(&FindTOTPSecretByUserId { user_id: user })
            .await
            .expect_err("no row must yield RowNotFound");
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_one_secret_when_the_user_has_several(pool: PgPool) {
        // BUG(pinned): nothing enforces one secret per user, and fetch_one happily
        // returns the first row of a multi-row result — a duplicated enrollment
        // silently picks an arbitrary secret instead of failing.
        let user = seed_user(&pool).await;
        seed_totp_secret(&pool, user, "SECRETFIRST00000").await;
        seed_totp_secret(&pool, user, "SECRETSECOND0000").await;

        let found = repo(&pool)
            .select_totp_secret_by_user_id(&FindTOTPSecretByUserId { user_id: user })
            .await
            .expect("fetch_one returns the first row without error");
        assert!(
            found.secret == "SECRETFIRST00000" || found.secret == "SECRETSECOND0000",
            "the winner among equal-created_at rows is unspecified, got: {}",
            found.secret
        );
    }
}
