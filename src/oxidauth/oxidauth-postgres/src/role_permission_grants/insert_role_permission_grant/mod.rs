use oxidauth_repository::role_permission_grants::insert_role_permission_grant::*;

use super::*;

#[async_trait]
impl InsertRolePermissionGrantQuery for PgRolePermissionGrantRepository {
    #[tracing::instrument(name = "insert_role_permission_grant_query", skip(self))]
    async fn insert_role_permission_grant(
        &self,
        params: &InsertRolePermissionGrant,
    ) -> Result<RolePermissionGrant, BoxedError> {
        let result = sqlx::query_as::<_, PgRolePermissionGrant>(include_str!(
            "./insert_role_permission_grant.sql"
        ))
        .bind(params.role_id)
        .bind(params.permission_id)
        .fetch_one(&self.db.write_pool())
        .await?;

        let role_permission_grant = result.into();

        Ok(role_permission_grant)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{assert_sql_state, seed_permission, seed_role},
    };

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_grant_a_permission_to_a_role(pool: PgPool) {
        let role = seed_role(&pool, "granted-role").await;
        let permission = seed_permission(&pool, "grant-test", "resource", "read").await;

        let repo = PgRolePermissionGrantRepository::new(Database::from_pool(pool.clone()));
        let grant = repo
            .insert_role_permission_grant(&InsertRolePermissionGrant {
                role_id: role,
                permission_id: permission,
            })
            .await
            .expect("insert should succeed");

        assert_eq!(grant.role_id, role);
        assert_eq!(grant.permission_id, permission);
        assert!(grant.created_at.timestamp() > 0);
        assert!(grant.updated_at >= grant.created_at);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM role_permission_grants WHERE role_id = $1 AND permission_id = $2",
        )
        .bind(role)
        .bind(permission)
        .fetch_one(&pool)
        .await
        .expect("count should run");
        assert_eq!(count, 1, "the returned grant must be persisted");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_a_duplicate_grant_pair(pool: PgPool) {
        let role = seed_role(&pool, "dup-grant-role").await;
        let permission = seed_permission(&pool, "grant-test", "resource", "read").await;

        let repo = PgRolePermissionGrantRepository::new(Database::from_pool(pool.clone()));
        let params = InsertRolePermissionGrant {
            role_id: role,
            permission_id: permission,
        };
        repo.insert_role_permission_grant(&params)
            .await
            .expect("first grant should succeed");

        let err = repo
            .insert_role_permission_grant(&params)
            .await
            .expect_err("the composite primary key must reject the duplicate pair");
        assert_sql_state(err, "23505");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM role_permission_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_unknown_role_or_permission_ids(pool: PgPool) {
        let role = seed_role(&pool, "fk-check-role").await;
        let permission = seed_permission(&pool, "grant-test", "resource", "read").await;

        let repo = PgRolePermissionGrantRepository::new(Database::from_pool(pool));

        let err = repo
            .insert_role_permission_grant(&InsertRolePermissionGrant {
                role_id: Uuid::new_v4(),
                permission_id: permission,
            })
            .await
            .expect_err("an unknown role id must violate the roles FK");
        assert_sql_state(err, "23503");

        let err = repo
            .insert_role_permission_grant(&InsertRolePermissionGrant {
                role_id: role,
                permission_id: Uuid::new_v4(),
            })
            .await
            .expect_err("an unknown permission id must violate the permissions FK");
        assert_sql_state(err, "23503");
    }
}
