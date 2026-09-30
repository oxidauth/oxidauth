use oxidauth_repository::authorities::insert_authority::*;

use super::*;

#[async_trait]
impl InsertAuthorityQuery for PgAuthorityRepository {
    #[tracing::instrument(name = "insert_authority_query", skip(self))]
    async fn insert_authority(&self, params: &CreateAuthority) -> Result<Authority, BoxedError> {
        let result = sqlx::query_as::<_, PgAuthority>(include_str!("./insert_authority.sql"))
            .bind(None::<Uuid>)
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
            .bind(
                params
                    .params
                    .clone()
                    .inner_value(),
            )
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
        authorities::{AuthoritySettings, NbfOffset, TotpSettings},
        jwt::EntitlementsEncoding,
    };
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;

    fn repo(pool: &PgPool) -> PgAuthorityRepository {
        PgAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    fn settings() -> AuthoritySettings {
        AuthoritySettings {
            jwt_ttl: Duration::from_secs(120),
            jwt_nbf_offset: NbfOffset::Disabled,
            refresh_token_ttl: Duration::from_secs(3600),
            totp: TotpSettings::Disabled,
            entitlements_encoding: EntitlementsEncoding::Txt,
        }
    }

    fn create_params(name: &str, client_key: Option<Uuid>) -> CreateAuthority {
        CreateAuthority {
            name: name.to_owned(),
            client_key,
            status: Some(AuthorityStatus::Enabled),
            strategy: AuthorityStrategy::UsernamePassword,
            settings: settings(),
            params: JsonValue::new(json!({ "issuer": name })),
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_round_trip_settings_and_encoding(pool: PgPool) {
        let client_key = Uuid::new_v4();
        let mut params = create_params("round_trip_authority", Some(client_key));
        params.status = Some(AuthorityStatus::Disabled);
        params.strategy = AuthorityStrategy::Oauth2;
        params
            .settings
            .entitlements_encoding = EntitlementsEncoding::Gz;

        let authority = repo(&pool)
            .insert_authority(&params)
            .await
            .expect("insert should succeed");

        assert_ne!(authority.id, Uuid::nil(), "id must be generated");
        assert_eq!(authority.name, "round_trip_authority");
        assert_eq!(authority.client_key, client_key);
        assert_eq!(authority.status.to_string(), "disabled");
        assert_eq!(authority.strategy.to_string(), "oauth2");
        assert_eq!(authority.settings.jwt_ttl, Duration::from_secs(120));
        assert!(
            matches!(
                authority
                    .settings
                    .jwt_nbf_offset,
                NbfOffset::Disabled
            ),
            "nbf offset must round-trip: {:?}",
            authority
                .settings
                .jwt_nbf_offset
        );
        assert!(
            matches!(
                authority
                    .settings
                    .entitlements_encoding,
                EntitlementsEncoding::Gz
            ),
            "encoding must round-trip as gz"
        );
        assert_eq!(
            &*authority.params,
            &json!({ "issuer": "round_trip_authority" })
        );

        // pin the enum strings as stored in the database
        let (status, strategy): (String, String) =
            sqlx::query_as("SELECT status, strategy FROM authorities WHERE id = $1")
                .bind(authority.id)
                .fetch_one(&pool)
                .await
                .expect("row should exist");
        assert_eq!(status, "disabled");
        assert_eq!(strategy, "oauth2");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_round_trip_enabled_totp_and_default_nbf_settings(pool: PgPool) {
        // `TotpSettings::Enabled` carries a `url::Url`; built via serde to avoid a
        // direct `url` dependency here.
        let totp: TotpSettings = serde_json::from_value(json!({
            "enabled": {
                "totp_ttl": { "secs": 90, "nanos": 0 },
                "webhook": "https://hooks.example.com/otp",
                "webhook_key": "s3cret",
            }
        }))
        .expect("totp settings");

        let mut params = create_params("totp_authority", Some(Uuid::new_v4()));
        params.settings.totp = totp;
        params.settings.jwt_nbf_offset = NbfOffset::default();

        let authority = repo(&pool)
            .insert_authority(&params)
            .await
            .expect("insert should succeed");

        match authority.settings.totp {
            TotpSettings::Enabled {
                totp_ttl,
                webhook,
                webhook_key,
            } => {
                assert_eq!(totp_ttl, Duration::from_secs(90));
                assert_eq!(webhook.as_str(), "https://hooks.example.com/otp");
                assert_eq!(webhook_key, "s3cret");
            },
            TotpSettings::Disabled => panic!("totp settings must decode as Enabled"),
        }

        match authority
            .settings
            .jwt_nbf_offset
        {
            NbfOffset::Enabled(offset) => assert_eq!(offset, Duration::from_secs(10)),
            NbfOffset::Disabled => panic!("nbf offset must decode as Enabled(10s)"),
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_duplicate_client_key(pool: PgPool) {
        let client_key = Uuid::new_v4();

        repo(&pool)
            .insert_authority(&create_params("first_authority", Some(client_key)))
            .await
            .expect("first insert should succeed");

        let err = repo(&pool)
            .insert_authority(&create_params("second_authority", Some(client_key)))
            .await
            .expect_err("duplicate client_key must fail");

        let debug = format!("{err:?}");
        assert!(debug.contains("duplicate key"), "unexpected error: {debug}");
        assert!(
            debug.contains("authorities_client_key_key"),
            "unexpected error: {debug}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_missing_status(pool: PgPool) {
        let mut params = create_params("statusless_authority", Some(Uuid::new_v4()));
        params.status = None;

        let err = repo(&pool)
            .insert_authority(&params)
            .await
            .expect_err("NULL status must violate the NOT NULL constraint");

        assert!(
            format!("{err:?}").contains("not-null constraint"),
            "expected NOT NULL violation, got: {err:?}"
        );
    }
}
