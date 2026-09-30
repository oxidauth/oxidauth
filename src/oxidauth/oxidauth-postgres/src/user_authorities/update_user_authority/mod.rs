use oxidauth_repository::user_authorities::update_user_authority::*;

use super::*;

#[async_trait]
impl UpdateUserAuthorityQuery for PgUserAuthorityRepository {
    #[tracing::instrument(name = "update_user_authority_query", skip(self))]
    async fn update_user_authority(
        &self,
        params: &UpdateUserAuthority,
    ) -> Result<UserAuthority, BoxedError> {
        let row = sqlx::query_as::<_, PgUserAuthority>(include_str!("./update_user_authority.sql"))
            .bind(params.user_id)
            .bind(params.authority_id)
            .bind(&*params.params)
            .fetch_one(&self.db.write_pool())
            .await?;

        let user_authority = row.into();

        Ok(user_authority)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::JsonValue;
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
    async fn it_should_overwrite_params(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "update-overwrite").await;
        let original = json!({ "p": "old-hash", "s": "old-salt" });
        seed_user_authority(
            &pool,
            user,
            authority,
            &format!("identifier-{user}"),
            original.clone(),
        )
        .await;

        let replacement = json!({ "p": "new-hash", "s": "new-salt", "rotated": true });
        let updated = repo(&pool)
            .update_user_authority(&UpdateUserAuthority {
                user_id: user,
                authority_id: authority,
                params: JsonValue::new(replacement.clone()),
            })
            .await
            .expect("update should succeed");

        assert_eq!(updated.user_id, user);
        assert_eq!(updated.authority_id, authority);
        // inner_value() consumes the JsonValue, so extract it once
        let updated_params = updated.params.inner_value();
        assert_eq!(updated_params, replacement, "params must be overwritten");
        assert_ne!(updated_params, original);

        let stored: serde_json::Value = sqlx::query_scalar(
            "SELECT params FROM user_authorities WHERE user_id = $1 AND authority_id = $2",
        )
        .bind(user)
        .bind(authority)
        .fetch_one(&pool)
        .await
        .expect("stored params");
        assert_eq!(stored, replacement);

        let (count, identifier): (i64, String) = sqlx::query_as(
            "SELECT COUNT(*)::BIGINT AS count, MAX(user_identifier) AS identifier \
             FROM user_authorities WHERE user_id = $1 AND authority_id = $2",
        )
        .bind(user)
        .bind(authority)
        .fetch_one(&pool)
        .await
        .expect("count should run");
        assert_eq!(count, 1, "update must not duplicate the row");
        assert_eq!(
            identifier,
            format!("identifier-{user}"),
            "user_identifier is not touched by update_user_authority"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_pair_is_missing(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "update-missing").await;

        let err = repo(&pool)
            .update_user_authority(&UpdateUserAuthority {
                user_id: user,
                authority_id: authority,
                params: JsonValue::new(json!({ "p": "anything" })),
            })
            .await
            .expect_err(
                "UPDATE ... RETURNING with no matching row yields RowNotFound, not a no-op",
            );
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_authorities")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0);
    }
}
