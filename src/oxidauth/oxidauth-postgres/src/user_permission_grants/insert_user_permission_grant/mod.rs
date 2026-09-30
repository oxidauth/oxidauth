use oxidauth_repository::user_permission_grants::insert_user_permission_grant::*;

use super::*;

#[async_trait]
impl InsertUserPermissionGrantQuery for PgUserPermissionGrantRepository {
    #[tracing::instrument(name = "insert_user_permission_grant_query", skip(self))]
    async fn insert_user_permission_grant(
        &self,
        params: &CreateUserPermissionGrant,
    ) -> Result<UserPermissionGrant, BoxedError> {
        let row = sqlx::query_as::<_, PgUserPermissionGrant>(include_str!(
            "./insert_user_permission_grant.sql"
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
        test_fixtures::{assert_sql_state, seed_permission, seed_user},
    };

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_grant_a_permission_to_a_user(pool: PgPool) {
        let user = seed_user(&pool).await;
        let permission = seed_permission(&pool, "user-perm", "resource", "read").await;

        let repo = PgUserPermissionGrantRepository::new(Database::from_pool(pool.clone()));
        let grant = repo
            .insert_user_permission_grant(&CreateUserPermissionGrant {
                user_id: user,
                permission_id: permission,
            })
            .await
            .expect("insert should succeed");

        assert_eq!(grant.user_id, user);
        assert_eq!(grant.permission_id, permission);
        assert!(grant.created_at.timestamp() > 0);
        assert!(grant.updated_at >= grant.created_at);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_permission_grants WHERE user_id = $1 AND permission_id = $2",
        )
        .bind(user)
        .bind(permission)
        .fetch_one(&pool)
        .await
        .expect("count should run");
        assert_eq!(count, 1, "the returned grant must be persisted");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_a_duplicate_grant_pair(pool: PgPool) {
        let user = seed_user(&pool).await;
        let permission = seed_permission(&pool, "user-perm", "resource", "read").await;

        let repo = PgUserPermissionGrantRepository::new(Database::from_pool(pool.clone()));
        let params = CreateUserPermissionGrant {
            user_id: user,
            permission_id: permission,
        };
        repo.insert_user_permission_grant(&params)
            .await
            .expect("first grant should succeed");

        let err = repo
            .insert_user_permission_grant(&params)
            .await
            .expect_err("the composite primary key must reject the duplicate pair");
        assert_sql_state(err, "23505");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_permission_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_unknown_user_or_permission_ids(pool: PgPool) {
        let user = seed_user(&pool).await;
        let permission = seed_permission(&pool, "user-perm", "resource", "read").await;

        let repo = PgUserPermissionGrantRepository::new(Database::from_pool(pool));

        let err = repo
            .insert_user_permission_grant(&CreateUserPermissionGrant {
                user_id: Uuid::new_v4(),
                permission_id: permission,
            })
            .await
            .expect_err("an unknown user id must violate the users FK");
        assert_sql_state(err, "23503");

        let err = repo
            .insert_user_permission_grant(&CreateUserPermissionGrant {
                user_id: user,
                permission_id: Uuid::new_v4(),
            })
            .await
            .expect_err("an unknown permission id must violate the permissions FK");
        assert_sql_state(err, "23503");
    }
}
