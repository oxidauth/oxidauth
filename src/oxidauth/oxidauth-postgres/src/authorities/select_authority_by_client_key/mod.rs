use oxidauth_kernel::authorities::find_authority_by_client_key::FindAuthorityByClientKey;
use oxidauth_repository::authorities::select_authority_by_client_key::*;

use super::*;

#[async_trait]
impl SelectAuthorityByClientKeyQuery for PgAuthorityRepository {
    #[tracing::instrument(name = "select_authority_by_client_key_query", skip(self))]
    async fn select_authority_by_client_key(
        &self,
        params: &FindAuthorityByClientKey,
    ) -> Result<Option<Authority>, BoxedError> {
        let result =
            sqlx::query_as::<_, PgAuthority>(include_str!("./select_authority_by_client_key.sql"))
                .bind(params.client_key)
                .fetch_optional(&self.db.read_pool())
                .await?;

        let authority = result
            .map(TryInto::try_into)
            .transpose()?;

        Ok(authority)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::{create_authority, default_authority_settings};

    fn repo(pool: &PgPool) -> PgAuthorityRepository {
        PgAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_authority(pool: &PgPool, name: &str, client_key: Uuid) {
        create_authority(
            pool,
            name,
            client_key,
            AuthorityStatus::Enabled,
            AuthorityStrategy::UsernamePassword,
            default_authority_settings(),
            json!({}),
        )
        .await;
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_authority_by_client_key(pool: PgPool) {
        let client_key = Uuid::new_v4();
        seed_authority(&pool, "client_key_lookup", client_key).await;

        let found = repo(&pool)
            .select_authority_by_client_key(&FindAuthorityByClientKey { client_key })
            .await
            .expect("query should succeed")
            .expect("authority should be found");

        assert_eq!(found.name, "client_key_lookup");
        assert_eq!(found.client_key, client_key);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_none_for_missing_client_key(pool: PgPool) {
        let absent = repo(&pool)
            .select_authority_by_client_key(&FindAuthorityByClientKey {
                client_key: Uuid::new_v4(),
            })
            .await
            .expect("query should succeed");
        assert!(absent.is_none(), "absent client_key must resolve to `None`");
    }
}
