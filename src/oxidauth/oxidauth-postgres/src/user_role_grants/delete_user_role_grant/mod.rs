use oxidauth_kernel::user_role_grants::delete_user_role_grant::DeleteUserRoleGrant;
use oxidauth_repository::user_role_grants::delete_user_role_grant::*;

use super::*;

#[async_trait]
impl DeleteUserRoleGrantQuery for PgUserRoleGrantRepository {
    #[tracing::instrument(name = "delete_user_role_grant_query", skip(self))]
    async fn delete_user_role_grant(
        &self,
        params: &DeleteUserRoleGrant,
    ) -> Result<UserRoleGrant, BoxedError> {
        let row =
            sqlx::query_as::<_, PgUserRoleGrant>(include_str!("./delete_user_role_grant.sql"))
                .bind(params.user_id)
                .bind(params.role_id)
                .fetch_one(&self.db.write_pool())
                .await?;

        let user_role_grant = row.into();

        Ok(user_role_grant)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_role, seed_user},
    };

    async fn seed_grant(pool: &PgPool) -> (Uuid, Uuid) {
        let user = seed_user(pool).await;
        let role = seed_role(pool, &format!("role-{}", Uuid::new_v4())).await;
        sqlx::query("INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)")
            .bind(user)
            .bind(role)
            .execute(pool)
            .await
            .expect("seed grant");
        (user, role)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_the_grant_pair_and_return_it(pool: PgPool) {
        let (user, role) = seed_grant(&pool).await;

        let repo = PgUserRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let deleted = repo
            .delete_user_role_grant(&DeleteUserRoleGrant {
                user_id: user,
                role_id: role,
            })
            .await
            .expect("delete should return the removed grant");

        assert_eq!(deleted.user_id, user);
        assert_eq!(deleted.role_id, role);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_role_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0, "the grant row must be gone");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_pair_does_not_exist(pool: PgPool) {
        // pinned: DELETE ... RETURNING + fetch_one means a missing pair is a
        // RowNotFound error, not a silent no-op
        let (user, role) = seed_grant(&pool).await;

        let repo = PgUserRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let err = repo
            .delete_user_role_grant(&DeleteUserRoleGrant {
                user_id: role,
                role_id: user,
            })
            .await
            .expect_err("deleting the swapped (non-existent) pair must error");

        assert!(
            matches!(
                err.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {err:?}"
        );

        // the real grant is untouched
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_role_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1);
    }
}
