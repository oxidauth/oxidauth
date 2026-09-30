use oxidauth_repository::authorities::select_authority_by_strategy::*;

use super::*;

#[async_trait]
impl SelectAuthorityByStrategyQuery for PgAuthorityRepository {
    #[tracing::instrument(name = "select_authority_by_strategy_query", skip(self))]
    async fn select_authority_by_strategy(
        &self,
        params: &FindAuthorityByStrategy,
    ) -> Result<Option<Authority>, BoxedError> {
        let result =
            sqlx::query_as::<_, PgAuthority>(include_str!("./select_authority_by_strategy.sql"))
                .bind(params.strategy.to_string())
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

    async fn seed_authority(pool: &PgPool, name: &str, strategy: AuthorityStrategy) {
        create_authority(
            pool,
            name,
            Uuid::new_v4(),
            AuthorityStatus::Enabled,
            strategy,
            default_authority_settings(),
            json!({}),
        )
        .await;
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_exactly_one_of_multiple_matches(pool: PgPool) {
        // the unique(strategy) constraint was dropped in migration
        // 20240523190325 — several authorities may share a strategy
        seed_authority(&pool, "oauth_one", AuthorityStrategy::Oauth2).await;
        seed_authority(&pool, "oauth_two", AuthorityStrategy::Oauth2).await;

        let found = repo(&pool)
            .select_authority_by_strategy(&FindAuthorityByStrategy {
                strategy: AuthorityStrategy::Oauth2,
            })
            .await
            .expect("query should succeed")
            .expect("an authority must be returned");

        // BUG(pinned): the SQL is `LIMIT 1` without ORDER BY — with multiple matches
        // exactly one arbitrary row is returned; the caller cannot tell that more
        // exist.
        assert!(
            found.name == "oauth_one" || found.name == "oauth_two",
            "unexpected authority returned: {}",
            found.name
        );
        assert_eq!(found.strategy.to_string(), "oauth2");
        assert_eq!(found.status.to_string(), "enabled");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_map_strategy_enum_and_return_none_when_absent(pool: PgPool) {
        seed_authority(&pool, "up_authority", AuthorityStrategy::UsernamePassword).await;

        let found = repo(&pool)
            .select_authority_by_strategy(&FindAuthorityByStrategy {
                strategy: AuthorityStrategy::UsernamePassword,
            })
            .await
            .expect("query should succeed")
            .expect("username_password authority must be found");
        assert_eq!(found.name, "up_authority");
        assert_eq!(found.strategy.to_string(), "username_password");

        // pin the stored enum string
        let stored: String = sqlx::query_scalar("SELECT strategy FROM authorities WHERE id = $1")
            .bind(found.id)
            .fetch_one(&pool)
            .await
            .expect("row should exist");
        assert_eq!(stored, "username_password");

        let absent = repo(&pool)
            .select_authority_by_strategy(&FindAuthorityByStrategy {
                strategy: AuthorityStrategy::SingleUseToken,
            })
            .await
            .expect("query should succeed");
        assert!(absent.is_none(), "absent strategy must resolve to `None`");
    }
}
