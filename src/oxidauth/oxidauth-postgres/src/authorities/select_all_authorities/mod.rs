use oxidauth_repository::authorities::select_all_authorities::*;

use super::*;

#[async_trait]
impl SelectAllAuthoritiesQuery for PgAuthorityRepository {
    #[tracing::instrument(name = "select_all_authorities_query", skip(self))]
    async fn select_all_authorities(
        &self,
        params: &ListAllAuthorities,
    ) -> Result<Vec<Authority>, BoxedError> {
        let result = sqlx::query_as::<_, PgAuthority>(include_str!("./select_all_authorities.sql"))
            .fetch_all(&self.db.read_pool())
            .await?;

        let authorities = result
            .into_iter()
            .map(|a| a.try_into())
            .collect::<Result<Vec<Authority>, BoxedError>>()?;

        Ok(authorities)
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
    use crate::test_fixtures::create_authority;

    fn repo(pool: &PgPool) -> PgAuthorityRepository {
        PgAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_authority(
        pool: &PgPool,
        name: &str,
        status: AuthorityStatus,
        strategy: AuthorityStrategy,
        settings: AuthoritySettings,
    ) {
        create_authority(
            pool,
            name,
            Uuid::new_v4(),
            status,
            strategy,
            settings,
            json!({ "seeded": name }),
        )
        .await;
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_all_authorities_with_decoded_settings(pool: PgPool) {
        seed_authority(
            &pool,
            "plain_authority",
            AuthorityStatus::Enabled,
            AuthorityStrategy::UsernamePassword,
            AuthoritySettings {
                jwt_ttl: Duration::from_secs(300),
                jwt_nbf_offset: NbfOffset::Disabled,
                refresh_token_ttl: Duration::from_secs(7200),
                totp: TotpSettings::Disabled,
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
        )
        .await;

        let totp: TotpSettings = serde_json::from_value(json!({
            "enabled": {
                "totp_ttl": { "secs": 45, "nanos": 0 },
                "webhook": "https://hooks.example.com/2fa",
                "webhook_key": "hk",
            }
        }))
        .expect("totp settings");

        seed_authority(
            &pool,
            "hardened_authority",
            AuthorityStatus::Disabled,
            AuthorityStrategy::Oauth2,
            AuthoritySettings {
                jwt_ttl: Duration::from_secs(60),
                jwt_nbf_offset: NbfOffset::default(),
                refresh_token_ttl: Duration::from_secs(604800),
                totp,
                entitlements_encoding: EntitlementsEncoding::Gz,
            },
        )
        .await;
        // Guard (OXA-000019 precedent): force the tie — seed rows tie on created_at
        // only if NOW() never ticks between INSERTs, and while updated_at equals
        // created_at an updated_at-first orderer is indistinguishable from the
        // correct one. Pin every row's created_at to the newest one, then drop the
        // max-id row's updated_at below that tie: `ORDER BY updated_at ASC, id ASC`
        // must then hoist the max-id row to the front — observably wrong — while
        // the correct orderer never reads updated_at and keeps it last.
        sqlx::query(
            "UPDATE authorities SET created_at = (SELECT max(created_at) FROM authorities), \
             updated_at = CASE WHEN id = (SELECT id FROM authorities ORDER BY id DESC LIMIT 1) \
             THEN (SELECT max(created_at) FROM authorities) - INTERVAL '1 hour' \
             ELSE (SELECT max(created_at) FROM authorities) END",
        )
        .execute(&pool)
        .await
        .expect("diverge one row's updated_at");

        let repo = repo(&pool);
        let authorities = repo
            .select_all_authorities(&ListAllAuthorities {})
            .await
            .expect("query should succeed");

        assert_eq!(authorities.len(), 2);

        assert!(
            authorities
                .windows(2)
                .all(|w| (w[0].created_at, w[0].id) <= (w[1].created_at, w[1].id)),
            "rows must be returned in (created_at, id) order"
        );

        let again = repo
            .select_all_authorities(&ListAllAuthorities {})
            .await
            .expect("repeated query should succeed");
        assert_eq!(
            authorities
                .iter()
                .map(|a| a.id)
                .collect::<Vec<_>>(),
            again
                .iter()
                .map(|a| a.id)
                .collect::<Vec<_>>(),
            "repeated calls must return identical id sequences"
        );

        let mut names: Vec<&str> = authorities
            .iter()
            .map(|a| a.name.as_str())
            .collect();
        names.sort_unstable();
        assert_eq!(names, vec!["hardened_authority", "plain_authority"]);

        let hardened = authorities
            .iter()
            .find(|a| a.name == "hardened_authority")
            .expect("hardened authority present");
        assert_eq!(hardened.status.to_string(), "disabled");
        assert_eq!(hardened.strategy.to_string(), "oauth2");
        assert!(matches!(
            hardened.settings.totp,
            TotpSettings::Enabled { .. }
        ));
        assert!(matches!(
            hardened
                .settings
                .entitlements_encoding,
            EntitlementsEncoding::Gz
        ));

        let plain = authorities
            .iter()
            .find(|a| a.name == "plain_authority")
            .expect("plain authority present");
        assert_eq!(plain.status.to_string(), "enabled");
        assert!(matches!(plain.settings.jwt_nbf_offset, NbfOffset::Disabled));
    }
}
