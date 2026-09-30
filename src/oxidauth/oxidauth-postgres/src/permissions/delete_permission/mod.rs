use oxidauth_kernel::permissions::{RawPermission, delete_permission::DeletePermission};
use oxidauth_repository::permissions::delete_permission::*;

use super::*;

#[async_trait]
impl DeletePermissionQuery for PgPermissionRepository {
    #[tracing::instrument(name = "delete_permission_query", skip(self))]
    async fn delete_permission(&self, params: &DeletePermission) -> Result<Permission, BoxedError> {
        let permission: RawPermission = (&params.permission).try_into()?;

        let result = sqlx::query_as::<_, PgPermission>(include_str!("delete_permission.sql"))
            .bind(&permission.realm)
            .bind(&permission.resource)
            .bind(&permission.action)
            .fetch_one(&self.db.write_pool())
            .await?;

        let permission = result.into();

        Ok(permission)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::permissions::{
        delete_permission::DeletePermission,
        find_permission_by_parts::FindPermissionByParts,
    };
    use oxidauth_repository::permissions::select_permission_by_parts::*;
    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_permission, seed_role, seed_role_permission_grant},
    };

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_delete_by_parts_and_error_on_a_second_delete(pool: PgPool) {
        let id = seed_permission(&pool, "gone", "stuff", "read").await;
        let repo = PgPermissionRepository::new(Database::from_pool(pool));

        let deleted = repo
            .delete_permission(&DeletePermission {
                permission: "gone:stuff:read".to_owned(),
            })
            .await
            .expect("delete should return the removed row");
        assert_eq!(deleted.id, id);
        assert_eq!(deleted.to_string(), "gone:stuff:read");

        let found = repo
            .select_permission_by_parts(&FindPermissionByParts {
                permission: "gone:stuff:read".to_owned(),
            })
            .await
            .expect("the parts query should succeed");
        assert!(found.is_none(), "the row must be gone after delete");

        // pinned: a second delete is an error, not a silent no-op
        let second = repo
            .delete_permission(&DeletePermission {
                permission: "gone:stuff:read".to_owned(),
            })
            .await
            .expect_err("deleting an already-deleted permission must error");
        assert!(
            matches!(
                second.downcast_ref::<sqlx::Error>(),
                Some(sqlx::Error::RowNotFound)
            ),
            "expected sqlx::Error::RowNotFound, got: {second:?}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_cascade_role_permission_grants_when_the_permission_is_deleted(pool: PgPool) {
        // FK role_permission_grants_permissions_fk is ON DELETE CASCADE:
        // deleting a referenced permission removes the grant rows instead of erroring
        let role = seed_role(&pool, "cascade-role").await;
        let permission = seed_permission(&pool, "cascade", "resource", "write").await;
        seed_role_permission_grant(&pool, role, permission).await;

        let repo = PgPermissionRepository::new(Database::from_pool(pool.clone()));
        let deleted = repo
            .delete_permission(&DeletePermission {
                permission: "cascade:resource:write".to_owned(),
            })
            .await
            .expect("deleting a referenced permission must succeed via FK ON DELETE CASCADE");
        assert_eq!(deleted.id, permission);

        let grants: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_permission_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(
            grants, 0,
            "the referencing grant row cascades away with the permission"
        );

        let roles: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE id = $1")
            .bind(role)
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(roles, 1, "the referencing role row is untouched");
    }
}
