use oxidauth_repository::user_authorities::select_user_authorities_by_user_id::*;

use super::*;

#[async_trait]
impl SelectUserAuthoritiesByUserIdQuery for PgUserAuthorityRepository {
    #[tracing::instrument(name = "select_user_authorities_by_user_id_query", skip(self))]
    async fn select_user_authorities_by_user_id(
        &self,
        params: &ListUserAuthoritiesByUserId,
    ) -> Result<Vec<UserAuthorityWithAuthority>, BoxedError> {
        let user_authorities = sqlx::query_as::<_, PgUserAuthorityWithAuthority>(include_str!(
            "./select_user_authorities_by_user_id.sql"
        ))
        .bind(params.user_id)
        .fetch_all(&self.db.read_pool())
        .await?
        .into_iter()
        .map(|u| u.try_into())
        .collect::<Result<Vec<UserAuthorityWithAuthority>, BoxedError>>()?;

        Ok(user_authorities)
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use serde_json::json;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_authority_with_settings, seed_user, seed_user_authority},
    };

    fn repo(pool: &PgPool) -> PgUserAuthorityRepository {
        PgUserAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_empty_for_a_user_without_authorities(pool: PgPool) {
        let user = seed_user(&pool).await;
        let _authority = seed_authority_with_settings(&pool, "no-link").await;

        let found = repo(&pool)
            .select_user_authorities_by_user_id(&ListUserAuthoritiesByUserId { user_id: user })
            .await
            .expect("query should succeed");

        assert!(found.is_empty(), "expected no rows, got: {found:?}");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_list_all_authorities_of_a_user(pool: PgPool) {
        let user = seed_user(&pool).await;
        let other_user = seed_user(&pool).await;
        let authority_a = seed_authority_with_settings(&pool, "list-a").await;
        let authority_b = seed_authority_with_settings(&pool, "list-b").await;
        seed_user_authority(
            &pool,
            user,
            authority_a,
            &format!("identifier-{user}-{authority_a}"),
            json!({}),
        )
        .await;
        seed_user_authority(
            &pool,
            user,
            authority_b,
            &format!("identifier-{user}-{authority_b}"),
            json!({}),
        )
        .await;
        seed_user_authority(
            &pool,
            other_user,
            authority_a,
            &format!("identifier-{other_user}-{authority_a}"),
            json!({}),
        )
        .await;

        let found = repo(&pool)
            .select_user_authorities_by_user_id(&ListUserAuthoritiesByUserId { user_id: user })
            .await
            .expect("query should succeed");

        assert_eq!(found.len(), 2, "both authorities must be listed");
        let ids: Vec<Uuid> = found
            .iter()
            .map(|f| f.authority.id)
            .collect();
        assert!(ids.contains(&authority_a) && ids.contains(&authority_b));
        assert!(
            found
                .iter()
                .all(|f| f.user_authority.user_id == user),
            "rows of other users must not leak into the list"
        );
        // the join decodes each authority alongside the link row
        assert!(
            found
                .iter()
                .all(|f| f.authority.settings.jwt_ttl == Duration::from_secs(120)),
            "joined authority settings must decode"
        );
    }
}
