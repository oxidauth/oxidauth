use async_trait::async_trait;
use oxidauth_kernel::error::BoxedError;
use oxidauth_repository::users::delete_user_by_id_query::*;

use super::*;

#[async_trait]
impl DeleteUserByIdQuery for PgUserRepository {
    #[tracing::instrument(name = "delete_user_by_id_query", skip(self))]
    async fn delete_user_by_id(&self, user_id: &DeleteUserById) -> Result<User, BoxedError> {
        let result = sqlx::query_as::<_, UserRow>(include_str!("./delete_user_by_id_query.sql"))
            .bind(user_id.user_id)
            .fetch_one(&self.db.write_pool())
            .await?;

        let user = result.try_into()?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::test_fixtures::{create_user, seed_role};

    fn repo(pool: &PgPool) -> PgUserRepository {
        PgUserRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_and_return_the_user(pool: PgPool) {
        let id = create_user(&pool, "delete_me").await;

        let deleted = repo(&pool)
            .delete_user_by_id(&DeleteUserById { user_id: id })
            .await
            .expect("delete should succeed");

        assert_eq!(deleted.id, id);
        assert_eq!(deleted.username, "delete_me");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("count query");
        assert_eq!(count, 0, "row must be gone");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_cascade_fk_referenced_grants_and_error_on_second_delete(pool: PgPool) {
        let user_id = create_user(&pool, "grantee").await;
        let role_id = seed_role(&pool, "delete_grant_role").await;

        sqlx::query("INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)")
            .bind(user_id)
            .bind(role_id)
            .execute(&pool)
            .await
            .expect("seed user_role_grant");

        // BUG(pinned): audit A1 expected an FK error for a user referenced by
        // user_role_grants, but the schema declares `ON DELETE CASCADE` — the delete
        // succeeds and silently drops the grant rows instead.
        let deleted = repo(&pool)
            .delete_user_by_id(&DeleteUserById { user_id })
            .await
            .expect("delete must succeed via ON DELETE CASCADE, not error");
        assert_eq!(deleted.id, user_id);

        let grants: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_role_grants WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&pool)
                .await
                .expect("count query");
        assert_eq!(grants, 0, "grants must be cascade-deleted with the user");

        // BUG(pinned): delete is NOT idempotent — a second delete of the same id
        // surfaces `RowNotFound` (fetch_one over `DELETE ... RETURNING *`).
        let err = repo(&pool)
            .delete_user_by_id(&DeleteUserById { user_id })
            .await
            .expect_err("second delete must error");
        assert!(
            format!("{err:?}").contains("RowNotFound"),
            "expected RowNotFound, got: {err:?}"
        );
    }
}
