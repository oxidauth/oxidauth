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

    async fn seed_authority(pool: &PgPool, name: &str, strategy: AuthorityStrategy) -> Authority {
        create_authority(
            pool,
            name,
            Uuid::new_v4(),
            AuthorityStatus::Enabled,
            strategy,
            default_authority_settings(),
            json!({}),
        )
        .await
    }

    /// A strategy may name several rows — `strategy`'s UNIQUE constraint went away
    /// in 20240523190325 — and bootstrap's fallback follows this query's answer
    /// when it has no pinned client key to ask about, which is where `oxidauth:admin`
    /// ends up. `ORDER BY created_at, id` is what turns that into the same row on
    /// every boot and on every replica.
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn multiple_matches_resolve_to_the_earliest_created(pool: PgPool) {
        // seeded newest-first so physical/insertion order cannot pass for the rule
        seed_authority(&pool, "oauth_newer", AuthorityStrategy::Oauth2).await;
        seed_authority(&pool, "oauth_oldest", AuthorityStrategy::Oauth2).await;
        seed_authority(&pool, "oauth_newest", AuthorityStrategy::Oauth2).await;

        // `insert_authority.sql` stamps NOW(), and a #[sqlx::test] transaction pins
        // that to one instant for every row — so the fixture ages them by hand,
        // otherwise the id tie-break is on trial instead of the ordering.
        for (name, age) in [
            ("oauth_oldest", 300),
            ("oauth_newer", 200),
            ("oauth_newest", 100),
        ] {
            sqlx::query("UPDATE authorities SET created_at = NOW() - ($1 || ' seconds')::interval WHERE name = $2")
                .bind(age.to_string())
                .bind(name)
                .execute(&pool)
                .await
                .expect("age the fixture row");
        }

        let found = repo(&pool)
            .select_authority_by_strategy(&FindAuthorityByStrategy {
                strategy: AuthorityStrategy::Oauth2,
            })
            .await
            .expect("query should succeed")
            .expect("an authority must be returned");

        assert_eq!(
            found.name, "oauth_oldest",
            "the ordered query answers with the earliest row, got {}",
            found.name
        );
        assert_eq!(found.strategy.to_string(), "oauth2");
        assert_eq!(found.status.to_string(), "enabled");
    }

    /// The `, id` half of the ordering. Rows minted together share a `NOW()`, and
    /// without a tiebreaker Postgres answers from whatever order the scan happens
    /// to produce — the case the bootstrap bug actually lived in, since a seeded
    /// install creates all of its authorities in one go. The expected row is the
    /// smallest id, taken from the rows the repository returned, not from the query
    /// under test.
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn a_created_at_tie_breaks_on_the_smallest_id(pool: PgPool) {
        let seeded = [
            seed_authority(&pool, "tied_one", AuthorityStrategy::Oauth2).await,
            seed_authority(&pool, "tied_two", AuthorityStrategy::Oauth2).await,
            seed_authority(&pool, "tied_three", AuthorityStrategy::Oauth2).await,
        ];

        sqlx::query("UPDATE authorities SET created_at = '2020-01-01T00:00:00Z'")
            .execute(&pool)
            .await
            .expect("tie every fixture row");
        let distinct_stamps: i64 =
            sqlx::query_scalar("SELECT count(DISTINCT created_at) FROM authorities")
                .fetch_one(&pool)
                .await
                .expect("count the distinct stamps");
        assert_eq!(distinct_stamps, 1, "the fixture must actually be tied");

        let expected = seeded
            .iter()
            .min_by_key(|authority| authority.id)
            .expect("three seeded rows");

        let found = repo(&pool)
            .select_authority_by_strategy(&FindAuthorityByStrategy {
                strategy: AuthorityStrategy::Oauth2,
            })
            .await
            .expect("query should succeed")
            .expect("an authority must be returned");

        assert_eq!(
            found.id, expected.id,
            "a tie must resolve to the smallest id, not to scan order ({} returned)",
            found.name
        );
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
