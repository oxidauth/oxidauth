use oxidauth_kernel::user_authorities::delete_user_authority::DeleteUserAuthority;
use oxidauth_repository::user_authorities::delete_user_authority::*;

use super::*;

#[async_trait]
impl DeleteUserAuthorityQuery for PgUserAuthorityRepository {
    #[tracing::instrument(name = "delete_user_authority_query", skip(self))]
    async fn delete_user_authority(
        &self,
        params: &DeleteUserAuthority,
    ) -> Result<UserAuthority, BoxedError> {
        let result =
            sqlx::query_as::<_, PgUserAuthority>(include_str!("./delete_user_authority.sql"))
                .bind(params.user_id)
                .bind(params.authority_id)
                .fetch_one(&self.db.write_pool())
                .await?;

        let user_authority = result.into();

        Ok(user_authority)
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;
    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_authority, seed_user, seed_user_authority},
    };

    fn repo(pool: &PgPool) -> PgUserAuthorityRepository {
        PgUserAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_and_return_the_removed_row(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "delete-happy").await;
        seed_user_authority(
            &pool,
            user,
            authority,
            &format!("identifier-{user}"),
            json!({}),
        )
        .await;

        let deleted = repo(&pool)
            .delete_user_authority(&DeleteUserAuthority {
                user_id: user,
                authority_id: authority,
            })
            .await
            .expect("delete should succeed");

        assert_eq!(deleted.user_id, user);
        assert_eq!(deleted.authority_id, authority);
        assert_eq!(deleted.user_identifier, format!("identifier-{user}"));

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_authorities")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_on_a_missing_pair_and_is_not_idempotent(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "delete-missing").await;

        let err = repo(&pool)
            .delete_user_authority(&DeleteUserAuthority {
                user_id: user,
                authority_id: authority,
            })
            .await
            .expect_err(
                "DELETE ... RETURNING with no matching row yields RowNotFound, not a no-op",
            );
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }
}
