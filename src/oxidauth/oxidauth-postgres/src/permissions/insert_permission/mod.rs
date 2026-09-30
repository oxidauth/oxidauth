use oxidauth_kernel::permissions::{RawPermission, create_permission::CreatePermission};
use oxidauth_repository::permissions::insert_permission::*;

use super::*;

#[async_trait]
impl InsertPermissionQuery for PgPermissionRepository {
    #[tracing::instrument(name = "insert_permission_query", skip(self))]
    async fn insert_permission(&self, params: &CreatePermission) -> Result<Permission, BoxedError> {
        let perm_string = &params.permission;
        let permission: RawPermission = perm_string.try_into()?;

        let result = sqlx::query_as::<_, PgPermission>(include_str!("insert_permission.sql"))
            .bind(None::<Uuid>)
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
        create_permission::CreatePermission,
        find_permission_by_parts::FindPermissionByParts,
    };
    use oxidauth_repository::permissions::select_permission_by_parts::*;
    use sqlx::PgPool;

    use super::*;
    use crate::{Database, test_fixtures::assert_sql_state};

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_insert_a_permission_and_return_the_stored_row(pool: PgPool) {
        let repo = PgPermissionRepository::new(Database::from_pool(pool));

        let created = repo
            .insert_permission(&CreatePermission {
                permission: "test:things:read".to_owned(),
            })
            .await
            .expect("insert_permission should succeed");

        assert_eq!(created.realm, "test");
        assert_eq!(created.resource, "things");
        assert_eq!(created.action, "read");
        assert_ne!(created.id, uuid::Uuid::nil());
        assert!(created.created_at.timestamp() > 0);
        assert!(created.updated_at >= created.created_at);

        // round-trip through the parts query
        let found = repo
            .select_permission_by_parts(&FindPermissionByParts {
                permission: "test:things:read".to_owned(),
            })
            .await
            .expect("select_permission_by_parts should succeed")
            .expect("the inserted permission should be found");
        assert_eq!(found.id, created.id);
        assert_eq!(found.to_string(), "test:things:read");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_a_duplicate_realm_resource_action(pool: PgPool) {
        let repo = PgPermissionRepository::new(Database::from_pool(pool.clone()));

        let params = CreatePermission {
            permission: "dup:parts:write".to_owned(),
        };
        repo.insert_permission(&params)
            .await
            .expect("first insert should succeed");

        let err = repo
            .insert_permission(&params)
            .await
            .expect_err("duplicate (realm, resource, action) must violate unique_grant_parts");
        assert_sql_state(err, "23505");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM permissions WHERE realm = $1 AND resource = $2 AND action = $3",
        )
        .bind("dup")
        .bind("parts")
        .bind("write")
        .fetch_one(&pool)
        .await
        .expect("count should run");
        assert_eq!(count, 1, "the rejected insert must not store a second row");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_malformed_permission_strings_before_touching_the_db(pool: PgPool) {
        let repo = PgPermissionRepository::new(Database::from_pool(pool.clone()));

        // fewer than three parts
        let err = repo
            .insert_permission(&CreatePermission {
                permission: "brokenpermission".to_owned(),
            })
            .await
            .expect_err("a permission needs realm:resource:action");
        assert!(
            err.to_string()
                .contains("all three parts"),
            "expected the RawPermission parse error, got: {err:?}"
        );

        // an empty part is rejected too
        let err = repo
            .insert_permission(&CreatePermission {
                permission: "realm::action".to_owned(),
            })
            .await
            .expect_err("an empty resource must be rejected");
        assert!(
            err.to_string()
                .contains("all three parts"),
            "expected the RawPermission parse error, got: {err:?}"
        );

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM permissions")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 0, "nothing should be stored for rejected strings");
    }
}
