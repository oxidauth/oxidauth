use oxidauth_repository::user_authorities::select_user_authority_by_user_id_and_authority_id::*;

use super::*;

#[async_trait]
impl SelectUserAuthorityByUserIdAndAuthorityIdQuery for PgUserAuthorityRepository {
    #[tracing::instrument(
        name = "select_user_authority_by_user_id_and_authority_id_query",
        skip(self)
    )]
    async fn select_user_authority_by_user_id_and_authority_id(
        &self,
        params: &FindUserAuthorityByUserIdAndAuthorityId,
    ) -> Result<UserAuthorityWithAuthority, BoxedError> {
        let result = sqlx::query_as::<_, PgUserAuthorityWithAuthority>(include_str!(
            "./select_user_authority_by_user_id_and_authority_id.sql"
        ))
        .bind(params.user_id)
        .bind(params.authority_id)
        .fetch_one(&self.db.read_pool())
        .await?;

        let user = result.try_into()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use oxidauth_kernel::authorities::{AuthorityStatus, AuthorityStrategy};
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_authority_with_settings, seed_user, seed_user_authority},
    };

    fn repo(pool: &PgPool) -> PgUserAuthorityRepository {
        PgUserAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_user_authority_with_joined_authority(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority_with_settings(&pool, "joined-authority").await;
        seed_user_authority(
            &pool,
            user,
            authority,
            &format!("identifier-{user}-{authority}"),
            json!({ "p": "argon2-hash" }),
        )
        .await;

        let found = repo(&pool)
            .select_user_authority_by_user_id_and_authority_id(
                &FindUserAuthorityByUserIdAndAuthorityId {
                    user_id: user,
                    authority_id: authority,
                },
            )
            .await
            .expect("query should succeed");

        assert_eq!(found.user_authority.user_id, user);
        assert_eq!(
            found
                .user_authority
                .authority_id,
            authority
        );
        assert_eq!(
            found
                .user_authority
                .user_identifier,
            format!("identifier-{user}-{authority}")
        );
        assert_eq!(
            found
                .user_authority
                .params
                .inner_value(),
            json!({ "p": "argon2-hash" })
        );

        assert_eq!(
            found.authority.id, authority,
            "the joined authority id is mapped"
        );
        assert!(
            found
                .authority
                .name
                .starts_with("joined-authority-")
        );
        assert!(matches!(found.authority.status, AuthorityStatus::Enabled));
        assert!(matches!(
            found.authority.strategy,
            AuthorityStrategy::UsernamePassword
        ));
        assert_eq!(
            found
                .authority
                .settings
                .jwt_ttl,
            Duration::from_secs(120),
            "authority settings must decode from the joined jsonb"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_pair_is_missing(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority_with_settings(&pool, "missing-pair").await;
        let other_authority = seed_authority_with_settings(&pool, "missing-pair-other").await;
        seed_user_authority(
            &pool,
            user,
            other_authority,
            &format!("identifier-{user}-{other_authority}"),
            json!({ "p": "argon2-hash" }),
        )
        .await;

        let err = repo(&pool)
            .select_user_authority_by_user_id_and_authority_id(
                &FindUserAuthorityByUserIdAndAuthorityId {
                    user_id: user,
                    authority_id: authority,
                },
            )
            .await
            .expect_err("a user attached to another authority must not match this pair");
        let sqlx_err = err
            .downcast_ref::<sqlx::Error>()
            .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
        assert!(
            matches!(sqlx_err, sqlx::Error::RowNotFound),
            "expected RowNotFound, got: {sqlx_err:?}"
        );
    }
}
