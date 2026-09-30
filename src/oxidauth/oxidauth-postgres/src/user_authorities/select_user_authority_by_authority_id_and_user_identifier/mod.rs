use oxidauth_kernel::user_authorities::UserAuthorityNotFoundError;

use oxidauth_repository::user_authorities::select_user_authority_by_authority_id_and_user_identifier::*;
use super::*;

#[async_trait]
impl SelectUserAuthorityByAuthorityIdAndUserIdentifierQuery for PgUserAuthorityRepository {
    #[tracing::instrument(
        name = "select_user_authority_by_authority_id_and_user_identifier_query",
        skip(self)
    )]
    async fn select_user_authority_by_authority_id_and_user_identifier(
        &self,
        params: &SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams,
    ) -> Result<UserAuthority, BoxedError> {
        let result = sqlx::query_as::<_, PgUserAuthority>(include_str!(
            "./select_user_authority_by_authority_id_and_user_identifier.sql"
        ))
        .bind(params.authority_id)
        .bind(&params.user_identifier)
        .fetch_one(&self.db.read_pool())
        .await
        .map_err(|err| -> BoxedError {
            match err {
                sqlx::Error::RowNotFound => {
                    UserAuthorityNotFoundError::new(
                        params.authority_id,
                        params.user_identifier.clone(),
                    )
                },
                _ => err.to_string().into(),
            }
        })?;
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

    /// The login-identifier lookup is deliberately a plain `user_identifier = $2`
    /// equality on the dedicated VARCHAR column — nothing is looked up inside the
    /// `params` jsonb (identifier extraction from request JSON is owned by the
    /// `UserIdentifierFromRequest` strategies). `fetch_one` surfaces exactly one row;
    /// an absent identifier maps `RowNotFound` to the typed `UserAuthorityNotFoundError`.
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_the_user_authority_by_username_identifier(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "identifier-login").await;
        seed_user_authority(
            &pool,
            user,
            authority,
            "alice@example.com",
            json!({ "p": "argon2id$v=19$m=19456,t=2,p=1$anp6dGVzdA" }),
        )
        .await;

        let found = repo(&pool)
            .select_user_authority_by_authority_id_and_user_identifier(
                &SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams {
                    authority_id: authority,
                    user_identifier: "alice@example.com".to_owned(),
                },
            )
            .await
            .expect("the login identifier lookup should match");

        assert_eq!(found.user_id, user);
        assert_eq!(found.authority_id, authority);
        assert_eq!(found.user_identifier, "alice@example.com");
        assert_eq!(
            found.params.inner_value(),
            json!({ "p": "argon2id$v=19$m=19456,t=2,p=1$anp6dGVzdA" }),
            "the stored argon2 hash must come back unchanged"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_map_absent_identifier_to_a_not_found_error(pool: PgPool) {
        let user = seed_user(&pool).await;
        let authority = seed_authority(&pool, "identifier-absent").await;
        let other_authority = seed_authority(&pool, "identifier-absent-other").await;
        seed_user_authority(
            &pool,
            user,
            authority,
            "alice@example.com",
            json!({ "p": "argon2id$v=19$m=19456,t=2,p=1$anp6dGVzdA" }),
        )
        .await;

        // identifier absent entirely for this authority
        let err = repo(&pool)
            .select_user_authority_by_authority_id_and_user_identifier(
                &SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams {
                    authority_id: authority,
                    user_identifier: "nobody@example.com".to_owned(),
                },
            )
            .await
            .expect_err("an absent identifier must not return Ok");
        // RowNotFound is mapped to a typed, downcastable error carrying the lookup key
        let message = err.to_string();
        assert!(
            message.starts_with("user authority not found"),
            "RowNotFound must render the not-found Display prefix, got: {message}"
        );
        let not_found = err
            .downcast_ref::<UserAuthorityNotFoundError>()
            .expect("the absent-identifier error must downcast to UserAuthorityNotFoundError");
        assert_eq!(not_found.authority_id, authority);
        assert_eq!(not_found.user_identifier, "nobody@example.com");

        // identifier exists, but under a different authority — the WHERE keeps scope to $1 AND $2
        let err = repo(&pool)
            .select_user_authority_by_authority_id_and_user_identifier(
                &SelectUserAuthorityByAuthorityIdAndUserIdentifierQueryParams {
                    authority_id: other_authority,
                    user_identifier: "alice@example.com".to_owned(),
                },
            )
            .await
            .expect_err("the identifier must be scoped to the requested authority");
        let message = err.to_string();
        assert!(
            message.starts_with("user authority not found"),
            "expected the mapped not-found error, got: {message}"
        );
        let not_found = err
            .downcast_ref::<UserAuthorityNotFoundError>()
            .expect("the cross-authority miss must also be a UserAuthorityNotFoundError");
        assert_eq!(not_found.authority_id, other_authority);
        assert_eq!(not_found.user_identifier, "alice@example.com");
    }
}
