use oxidauth_repository::authorities::update_authority::*;

use super::*;

#[async_trait]
impl UpdateAuthorityQuery for PgAuthorityRepository {
    #[tracing::instrument(name = "update_authority_query", skip(self))]
    async fn update_authority(&self, params: &UpdateAuthority) -> Result<Authority, BoxedError> {
        let result = sqlx::query_as::<_, PgAuthority>(include_str!("./update_authority.sql"))
            .bind(params.id)
            .bind(&params.name)
            .bind(params.client_key)
            .bind(
                params
                    .status
                    .as_ref()
                    .map(|s| s.to_string()),
            )
            .bind(params.strategy.to_string())
            .bind(serde_json::to_value(&params.settings)?)
            .bind(&params.params)
            .fetch_one(&self.db.write_pool())
            .await?;

        let authority = result.try_into()?;

        Ok(authority)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use oxidauth_kernel::{
        authorities::{
            AuthoritySettings,
            NbfOffset,
            TotpSettings,
            find_authority_by_id::FindAuthorityById,
        },
        jwt::EntitlementsEncoding,
    };
    use oxidauth_repository::authorities::select_authority_by_id::*;
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::create_authority;

    fn repo(pool: &PgPool) -> PgAuthorityRepository {
        PgAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    fn settings(jwt_secs: u64, encoding: EntitlementsEncoding) -> AuthoritySettings {
        AuthoritySettings {
            jwt_ttl: Duration::from_secs(jwt_secs),
            jwt_nbf_offset: NbfOffset::Disabled,
            refresh_token_ttl: Duration::from_secs(3600),
            totp: TotpSettings::Disabled,
            entitlements_encoding: encoding,
        }
    }

    async fn seed_authority(pool: &PgPool, name: &str, client_key: Uuid) -> Authority {
        create_authority(
            pool,
            name,
            client_key,
            AuthorityStatus::Disabled,
            AuthorityStrategy::Oauth2,
            settings(120, EntitlementsEncoding::Txt),
            json!({ "v": 1 }),
        )
        .await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_overwrite_every_column(pool: PgPool) {
        let old_client_key = Uuid::new_v4();
        let seeded = seed_authority(&pool, "overwrite_me", old_client_key).await;
        let new_client_key = Uuid::new_v4();

        let params = UpdateAuthority {
            id: Some(seeded.id),
            name: "overwritten".to_owned(),
            client_key: Some(new_client_key),
            status: Some(AuthorityStatus::Enabled),
            strategy: AuthorityStrategy::UsernamePassword,
            settings: settings(60, EntitlementsEncoding::Gz),
            params: json!({ "v": 2 }),
        };

        let updated = repo(&pool)
            .update_authority(&params)
            .await
            .expect("update should succeed");

        assert_eq!(updated.id, seeded.id);
        assert_eq!(updated.name, "overwritten");
        assert_eq!(updated.client_key, new_client_key);
        assert_eq!(updated.status.to_string(), "enabled");
        assert_eq!(updated.strategy.to_string(), "username_password");
        assert_eq!(updated.settings.jwt_ttl, Duration::from_secs(60));
        assert!(matches!(
            updated
                .settings
                .entitlements_encoding,
            EntitlementsEncoding::Gz
        ));
        assert_eq!(&*updated.params, &json!({ "v": 2 }));

        let reread = repo(&pool)
            .select_authority_by_id(&FindAuthorityById {
                authority_id: seeded.id,
            })
            .await
            .expect("reread should succeed");
        assert_eq!(reread.name, "overwritten");
        assert_eq!(reread.client_key, new_client_key);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_null_status_overwrite(pool: PgPool) {
        // BUG(pinned): update_authority.sql writes NULL for `status: None`
        // instead of keeping the stored value — NOT NULL columns cannot be left
        // untouched by an update at the SQL level.
        let seeded = seed_authority(&pool, "status_overwrite", Uuid::new_v4()).await;

        let params = UpdateAuthority {
            id: Some(seeded.id),
            name: "status_overwrite".to_owned(),
            client_key: Some(Uuid::new_v4()),
            status: None,
            strategy: AuthorityStrategy::Oauth2,
            settings: settings(120, EntitlementsEncoding::Txt),
            params: json!({}),
        };

        let err = repo(&pool)
            .update_authority(&params)
            .await
            .expect_err("NULL status must violate the NOT NULL constraint");
        assert!(
            format!("{err:?}").contains("not-null constraint"),
            "expected NOT NULL violation, got: {err:?}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_write_null_client_key_that_the_read_path_cannot_decode(pool: PgPool) {
        // BUG(pinned): `client_key: None` overwrites the column with NULL (the column
        // default does not apply when the column is listed explicitly). The write
        // lands, but the repository cannot decode NULL back into `Uuid`, so the
        // update call errors *after* the row is already corrupted.
        let seeded = seed_authority(&pool, "null_client_key", Uuid::new_v4()).await;

        let params = UpdateAuthority {
            id: Some(seeded.id),
            name: "null_client_key".to_owned(),
            client_key: None,
            status: Some(AuthorityStatus::Enabled),
            strategy: AuthorityStrategy::Oauth2,
            settings: settings(120, EntitlementsEncoding::Txt),
            params: json!({}),
        };

        let err = repo(&pool)
            .update_authority(&params)
            .await
            .expect_err("NULL client_key cannot decode into Uuid");
        assert!(
            format!("{err:?}").contains("Decode"),
            "expected a decode error, got: {err:?}"
        );

        let stored: Option<Uuid> =
            sqlx::query_scalar("SELECT client_key FROM authorities WHERE id = $1")
                .bind(seeded.id)
                .fetch_one(&pool)
                .await
                .expect("row should exist");
        // BUG(pinned): the row is corrupted by the write: `client_key` is now NULL,
        // and the read path can never decode it back into an `Authority`.
        assert!(stored.is_none(), "pinned: client_key overwritten with NULL");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_on_missing_authority(pool: PgPool) {
        let params = UpdateAuthority {
            id: Some(Uuid::new_v4()),
            name: "ghost".to_owned(),
            client_key: Some(Uuid::new_v4()),
            status: Some(AuthorityStatus::Enabled),
            strategy: AuthorityStrategy::Oauth2,
            settings: settings(120, EntitlementsEncoding::Txt),
            params: json!({}),
        };

        let err = repo(&pool)
            .update_authority(&params)
            .await
            .expect_err("missing row must surface as an error");
        assert!(
            format!("{err:?}").contains("RowNotFound"),
            "expected RowNotFound, got: {err:?}"
        );
    }
}
