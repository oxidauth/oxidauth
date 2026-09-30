use oxidauth_repository::authorities::delete_authority::*;

use super::*;

#[async_trait]
impl DeleteAuthorityQuery for PgAuthorityRepository {
    #[tracing::instrument(name = "delete_authority_query", skip(self))]
    async fn delete_authority(&self, params: &DeleteAuthority) -> Result<Authority, BoxedError> {
        let result = sqlx::query_as::<_, PgAuthority>(include_str!("./delete_authority.sql"))
            .bind(params.authority_id)
            .fetch_one(&self.db.write_pool())
            .await?;

        let authority = result.try_into()?;

        Ok(authority)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::{create_authority, default_authority_settings, seed_user_named};

    fn repo(pool: &PgPool) -> PgAuthorityRepository {
        PgAuthorityRepository::new(Database::from_pool(pool.clone()))
    }

    async fn seed_authority(pool: &PgPool, name: &str) -> Uuid {
        create_authority(
            pool,
            name,
            Uuid::new_v4(),
            AuthorityStatus::Enabled,
            AuthorityStrategy::UsernamePassword,
            default_authority_settings(),
            json!({}),
        )
        .await
        .id
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_and_return_the_authority(pool: PgPool) {
        let id = seed_authority(&pool, "delete_me_authority").await;

        let deleted = repo(&pool)
            .delete_authority(&DeleteAuthority { authority_id: id })
            .await
            .expect("delete should succeed");

        assert_eq!(deleted.id, id);
        assert_eq!(deleted.name, "delete_me_authority");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM authorities WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("count query");
        assert_eq!(count, 0, "row must be gone");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_cascade_user_links_and_error_on_second_delete(pool: PgPool) {
        let authority_id = seed_authority(&pool, "linked_authority").await;
        let user_id = seed_user_named(&pool, "linked_user").await;

        sqlx::query(
            "INSERT INTO user_authorities (user_id, authority_id, user_identifier, params) \
             VALUES ($1, $2, $3, '{}'::jsonb)",
        )
        .bind(user_id)
        .bind(authority_id)
        .bind("linked_user")
        .execute(&pool)
        .await
        .expect("seed user_authority");

        // BUG(pinned): audit A2 expected an FK error for an authority referenced by
        // user_authorities, but the schema declares `ON DELETE CASCADE` — the delete
        // succeeds and silently drops the user link rows instead.
        let deleted = repo(&pool)
            .delete_authority(&DeleteAuthority { authority_id })
            .await
            .expect("delete must succeed via ON DELETE CASCADE, not error");
        assert_eq!(deleted.id, authority_id);

        let links: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_authorities WHERE authority_id = $1")
                .bind(authority_id)
                .fetch_one(&pool)
                .await
                .expect("count query");
        assert_eq!(links, 0, "user links must be cascade-deleted");

        let err = repo(&pool)
            .delete_authority(&DeleteAuthority { authority_id })
            .await
            .expect_err("second delete must error (not idempotent)");
        assert!(
            format!("{err:?}").contains("RowNotFound"),
            "expected RowNotFound, got: {err:?}"
        );
    }
}
