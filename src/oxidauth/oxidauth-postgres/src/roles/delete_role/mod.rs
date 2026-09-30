use oxidauth_kernel::roles::delete_role::DeleteRole;
use oxidauth_repository::roles::delete_role::*;

use super::*;

#[async_trait]
impl DeleteRoleQuery for PgRoleRepository {
    #[tracing::instrument(name = "delete_role_query", skip(self))]
    async fn delete_role(&self, params: &DeleteRole) -> Result<Role, BoxedError> {
        let result = sqlx::query_as::<_, PgRole>(include_str!("./delete_role.sql"))
            .bind(params.role_id)
            .fetch_one(&self.db.write_pool())
            .await?;

        let role = result.into();

        Ok(role)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::roles::{create_role::CreateRole, delete_role::DeleteRole};
    use oxidauth_repository::roles::insert_role::*;
    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{
            seed_permission,
            seed_role,
            seed_role_permission_grant,
            seed_role_role_grant,
        },
    };

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_the_role_and_return_the_removed_row(pool: PgPool) {
        let repo = PgRoleRepository::new(Database::from_pool(pool));

        let created = repo
            .insert_role(&CreateRole {
                name: "doomed-role".to_owned(),
            })
            .await
            .expect("seed insert should succeed");

        let deleted = repo
            .delete_role(&DeleteRole {
                role_id: created.id,
            })
            .await
            .expect("delete should return the removed row");
        assert_eq!(deleted.id, created.id);
        assert_eq!(deleted.name, "doomed-role");

        // a second delete is an error (DELETE ... RETURNING finds no row), not a no-op
        let second = repo
            .delete_role(&DeleteRole {
                role_id: created.id,
            })
            .await
            .expect_err("deleting an already-deleted role must error");
        assert!(
            matches!(
                second.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {second:?}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_cascade_role_permission_grants_when_the_role_is_deleted(pool: PgPool) {
        let role = seed_role(&pool, "grants-cascade-role").await;
        let permission = seed_permission(&pool, "cascade", "resource", "read").await;
        seed_role_permission_grant(&pool, role, permission).await;

        let repo = PgRoleRepository::new(Database::from_pool(pool.clone()));
        repo.delete_role(&DeleteRole { role_id: role })
            .await
            .expect("deleting a referenced role must succeed via FK ON DELETE CASCADE");

        let grants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_permission_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(
            grants, 0,
            "the referencing grant row cascades away with the role"
        );

        let permissions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM permissions")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(permissions, 1, "the referenced permission row is untouched");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_cascade_role_role_grants_when_the_parent_is_deleted(pool: PgPool) {
        let parent = seed_role(&pool, "cycle-parent").await;
        let child = seed_role(&pool, "cycle-child").await;
        seed_role_role_grant(&pool, parent, child).await;

        let repo = PgRoleRepository::new(Database::from_pool(pool.clone()));
        repo.delete_role(&DeleteRole { role_id: parent })
            .await
            .expect("deleting a parent role that is still referenced must succeed via cascade");

        let grants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_role_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(
            grants, 0,
            "the referencing edge cascades away with the parent"
        );

        let children: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE id = $1")
            .bind(child)
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(children, 1, "the child role survives its parent's deletion");
    }
}
