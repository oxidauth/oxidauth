use oxidauth_kernel::user_permission_grants::delete_user_permission_grant::DeleteUserPermissionGrant;
use oxidauth_repository::user_permission_grants::delete_user_permission_grant::*;

use super::*;

#[async_trait]
impl DeleteUserPermissionGrantQuery for PgUserPermissionGrantRepository {
    #[tracing::instrument(name = "delete_user_permission_grant_query", skip(self))]
    async fn delete_user_permission_grant(
        &self,
        params: &DeleteUserPermissionGrant,
    ) -> Result<UserPermissionGrant, BoxedError> {
        let row = sqlx::query_as::<_, PgUserPermissionGrant>(include_str!(
            "./delete_user_permission_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.permission_id)
        .fetch_one(&self.db.write_pool())
        .await?;

        let user_permission_grant = row.into();

        Ok(user_permission_grant)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_permission, seed_user, seed_user_permission_grant},
    };

    async fn seed_grant(pool: &PgPool) -> (Uuid, Uuid) {
        let user = seed_user(pool).await;
        let permission = seed_permission(pool, "del-test", "resource", "read").await;
        seed_user_permission_grant(pool, user, permission).await;
        (user, permission)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_the_grant_pair_and_return_it(pool: PgPool) {
        let (user, permission) = seed_grant(&pool).await;

        let repo = PgUserPermissionGrantRepository::new(Database::from_pool(pool.clone()));
        let deleted = repo
            .delete_user_permission_grant(&DeleteUserPermissionGrant {
                user_id: user,
                permission_id: permission,
            })
            .await
            .expect("delete should return the removed grant");

        assert_eq!(deleted.user_id, user);
        assert_eq!(deleted.permission_id, permission);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_permission_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0, "the grant row must be gone");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_pair_does_not_exist(pool: PgPool) {
        // pinned: DELETE ... RETURNING + fetch_one means a missing pair is a
        // RowNotFound error, not a silent no-op
        let (user, _permission) = seed_grant(&pool).await;

        let repo = PgUserPermissionGrantRepository::new(Database::from_pool(pool.clone()));
        let err = repo
            .delete_user_permission_grant(&DeleteUserPermissionGrant {
                user_id: user,
                permission_id: Uuid::new_v4(),
            })
            .await
            .expect_err("deleting a non-existent pair must error");

        assert!(
            matches!(
                err.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {err:?}"
        );

        // the real grant is untouched
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_permission_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1);
    }
}
