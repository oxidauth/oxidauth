use oxidauth_kernel::role_role_grants::delete_role_role_grant::DeleteRoleRoleGrant;
use oxidauth_repository::role_role_grants::delete_role_role_grant::*;

use super::*;

#[async_trait]
impl DeleteRoleRoleGrantQuery for PgRoleRoleGrantRepository {
    #[tracing::instrument(name = "delete_role_role_grant_query", skip(self))]
    async fn delete_role_role_grant(
        &self,
        params: &DeleteRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError> {
        let result =
            sqlx::query_as::<_, PgRoleRoleGrant>(include_str!("./delete_role_role_grant.sql"))
                .bind(params.parent_id)
                .bind(params.child_id)
                .fetch_one(&self.db.write_pool())
                .await?;

        let role_role_grant = result.into();

        Ok(role_role_grant)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_role, seed_role_role_grant},
    };

    async fn seed_edge(pool: &PgPool) -> (Uuid, Uuid) {
        let parent = seed_role(pool, &format!("parent-{}", Uuid::new_v4())).await;
        let child = seed_role(pool, &format!("child-{}", Uuid::new_v4())).await;
        seed_role_role_grant(pool, parent, child).await;
        (parent, child)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_the_edge_and_return_it(pool: PgPool) {
        let (parent, child) = seed_edge(&pool).await;

        let repo = PgRoleRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let deleted = repo
            .delete_role_role_grant(&DeleteRoleRoleGrant {
                parent_id: parent,
                child_id: child,
            })
            .await
            .expect("delete should return the removed edge");

        assert_eq!(deleted.parent_id, parent);
        assert_eq!(deleted.child_id, child);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_role_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0, "the edge must be gone");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_when_the_pair_does_not_exist(pool: PgPool) {
        // pinned: DELETE ... RETURNING + fetch_one means a missing pair is a
        // RowNotFound error, not a silent no-op
        let (parent, child) = seed_edge(&pool).await;

        let repo = PgRoleRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let err = repo
            .delete_role_role_grant(&DeleteRoleRoleGrant {
                parent_id: child,
                child_id: parent,
            })
            .await
            .expect_err("deleting the reverse (non-existent) edge must error");

        assert!(
            matches!(
                err.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {err:?}"
        );

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_role_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1, "the real edge is untouched");
    }
}
