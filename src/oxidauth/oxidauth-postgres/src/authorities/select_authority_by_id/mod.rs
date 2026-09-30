use oxidauth_repository::authorities::select_authority_by_id::*;

use super::*;

#[async_trait]
impl SelectAuthorityByIdQuery for PgAuthorityRepository {
    #[tracing::instrument(name = "select_authority_by_id_query", skip(self))]
    async fn select_authority_by_id(
        &self,
        params: &FindAuthorityById,
    ) -> Result<Authority, BoxedError> {
        let result = sqlx::query_as::<_, PgAuthority>(include_str!("./query_authority_by_id.sql"))
            .bind(params.authority_id)
            .fetch_one(&self.db.read_pool())
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
            create_authority::CreateAuthority,
        },
        jwt::EntitlementsEncoding,
    };
    use oxidauth_repository::authorities::insert_authority::*;
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;

    fn repo(pool: &PgPool) -> PgAuthorityRepository {
        PgAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_authority_by_id(pool: PgPool) {
        let client_key = Uuid::new_v4();
        let params = CreateAuthority {
            name: "by_id_authority".to_owned(),
            client_key: Some(client_key),
            status: Some(AuthorityStatus::Enabled),
            strategy: AuthorityStrategy::SingleUseToken,
            settings: AuthoritySettings {
                jwt_ttl: Duration::from_secs(30),
                jwt_nbf_offset: NbfOffset::Disabled,
                refresh_token_ttl: Duration::from_secs(600),
                totp: TotpSettings::Disabled,
                entitlements_encoding: EntitlementsEncoding::Txt,
            },
            params: JsonValue::new(json!({ "nonce": "abc" })),
        };

        let created = repo(&pool)
            .insert_authority(&params)
            .await
            .expect("seed authority should succeed");

        let found = repo(&pool)
            .select_authority_by_id(&FindAuthorityById {
                authority_id: created.id,
            })
            .await
            .expect("query should succeed");

        assert_eq!(found.id, created.id);
        assert_eq!(found.name, "by_id_authority");
        assert_eq!(found.client_key, client_key);
        assert_eq!(found.status.to_string(), "enabled");
        assert_eq!(found.strategy.to_string(), "single_use_token");
        assert_eq!(found.settings.jwt_ttl, Duration::from_secs(30));
        assert_eq!(&*found.params, &json!({ "nonce": "abc" }));
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_on_missing_authority(pool: PgPool) {
        let err = repo(&pool)
            .select_authority_by_id(&FindAuthorityById {
                authority_id: Uuid::new_v4(),
            })
            .await
            .expect_err("missing row must surface as an error");

        assert!(
            format!("{err:?}").contains("RowNotFound"),
            "expected RowNotFound, got: {err:?}"
        );
    }
}
