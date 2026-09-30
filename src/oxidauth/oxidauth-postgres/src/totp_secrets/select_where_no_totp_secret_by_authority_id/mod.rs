use oxidauth_repository::totp_secrets::select_where_no_totp_secret_by_authority_id::*;
use sqlx::PgConnection;

use super::*;

#[async_trait]
impl SelectWhereNoTotpSecretByAuthorityIdQuery for PgTotpSecretRepository {
    #[tracing::instrument(name = "select_where_no_totp_secret_by_authority_id", skip(self))]
    async fn select_where_no_totp_secret_by_authority_id(
        &self,
        params: &SelectWhereNoTotpSecretByAuthorityIdParams,
    ) -> Result<Vec<Uuid>, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        select_where_no_totp_secret_by_authority_id(&mut conn, params.authority_id).await
    }
}

pub async fn select_where_no_totp_secret_by_authority_id(
    conn: &mut PgConnection,
    authority_id: Uuid,
) -> Result<Vec<Uuid>, BoxedError> {
    let result = sqlx::query_as::<_, (Uuid,)>(include_str!(
        "./select_where_no_totp_secret_by_authority_id.sql"
    ))
    .bind(authority_id)
    .fetch_all(conn)
    .await?;

    let result = result
        .into_iter()
        .map(|row| row.0)
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_authority, seed_totp_secret, seed_user},
    };

    fn repo(pool: &PgPool) -> PgTotpSecretRepository {
        PgTotpSecretRepository::new(Database::from_pool(pool.clone()))
    }

    async fn attach(pool: &PgPool, user: Uuid, authority: Uuid) {
        sqlx::query(
            "INSERT INTO user_authorities (user_id, authority_id, user_identifier) \
                      VALUES ($1, $2, $3)",
        )
        .bind(user)
        .bind(authority)
        .bind(format!("identifier-{user}-{authority}"))
        .execute(pool)
        .await
        .expect("attach user authority");
    }

    async fn pending(pool: &PgPool, authority: Uuid) -> Vec<Uuid> {
        let mut ids = repo(&pool)
            .select_where_no_totp_secret_by_authority_id(
                &SelectWhereNoTotpSecretByAuthorityIdParams {
                    authority_id: authority,
                },
            )
            .await
            .expect("query should succeed");
        ids.sort();
        ids
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_exclude_users_that_already_have_a_secret(pool: PgPool) {
        let authority = seed_authority(&pool, "rollout-a").await;
        let user_plain = seed_user(&pool).await;
        let user_with_secret = seed_user(&pool).await;
        let user_plain_b = seed_user(&pool).await;
        attach(&pool, user_plain, authority).await;
        attach(&pool, user_with_secret, authority).await;
        attach(&pool, user_plain_b, authority).await;
        seed_totp_secret(&pool, user_with_secret, "JBSWY3DPEHPK3PXP").await;

        let mut expected = vec![user_plain, user_plain_b];
        expected.sort();
        assert_eq!(
            pending(&pool, authority).await,
            expected,
            "the enrolled user must drop out of the rollout list"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_scope_to_the_requested_authority_only(pool: PgPool) {
        let authority_a = seed_authority(&pool, "rollout-a2").await;
        let authority_b = seed_authority(&pool, "rollout-b2").await;
        let user_a = seed_user(&pool).await;
        let user_b = seed_user(&pool).await;
        let user_b_enrolled = seed_user(&pool).await;
        attach(&pool, user_a, authority_a).await;
        attach(&pool, user_b, authority_b).await;
        attach(&pool, user_b_enrolled, authority_b).await;
        // the secret is attached to a user that belongs to authority B only
        seed_totp_secret(&pool, user_b_enrolled, "JBSWY3DPEHPK3PXP").await;

        // authority A's list is exactly its own member …
        assert_eq!(pending(&pool, authority_a).await, vec![user_a]);
        // … authority B's list excludes its enrolled member
        assert_eq!(pending(&pool, authority_b).await, vec![user_b]);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_exclude_a_multi_authority_user_globally(pool: PgPool) {
        // The LEFT JOIN matches on user_id only — an enrollment excludes the user
        // from EVERY authority's rollout list, not just one. If the join were ever
        // scoped per authority, this test is where it fails.
        let authority_a = seed_authority(&pool, "rollout-multi-a").await;
        let authority_b = seed_authority(&pool, "rollout-multi-b").await;
        let shared_enrolled = seed_user(&pool).await;
        let plain_a = seed_user(&pool).await;
        let plain_b = seed_user(&pool).await;
        attach(&pool, shared_enrolled, authority_a).await;
        attach(&pool, shared_enrolled, authority_b).await;
        attach(&pool, plain_a, authority_a).await;
        attach(&pool, plain_b, authority_b).await;
        seed_totp_secret(&pool, shared_enrolled, "JBSWY3DPEHPK3PXP").await;

        assert_eq!(
            pending(&pool, authority_a).await,
            vec![plain_a],
            "the shared user's secret drops them from authority A's list"
        );
        assert_eq!(
            pending(&pool, authority_b).await,
            vec![plain_b],
            "… and from authority B's list too (user_id-only join)"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_empty_for_an_authority_without_users(pool: PgPool) {
        let authority = seed_authority(&pool, "rollout-empty").await;
        assert!(
            pending(&pool, authority)
                .await
                .is_empty()
        );
    }
}
