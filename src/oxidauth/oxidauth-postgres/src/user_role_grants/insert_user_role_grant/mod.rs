use oxidauth_repository::user_role_grants::insert_user_role_grant::*;

use super::*;

#[async_trait]
impl InsertUserRoleGrantQuery for PgUserRoleGrantRepository {
    #[tracing::instrument(name = "insert_user_role_grant_query", skip(self))]
    async fn insert_user_role_grant(
        &self,
        params: &CreateUserRoleGrant,
    ) -> Result<UserRoleGrant, BoxedError> {
        let row =
            sqlx::query_as::<_, PgUserRoleGrant>(include_str!("./insert_user_role_grant.sql"))
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
        test_fixtures::{assert_sql_state, seed_role, seed_user},
    };

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_grant_a_role_to_a_user(pool: PgPool) {
        let user = seed_user(&pool).await;
        let role = seed_role(&pool, "assigned-role").await;

        let repo = PgUserRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let grant = repo
            .insert_user_role_grant(&CreateUserRoleGrant {
                user_id: user,
                role_id: role,
            })
            .await
            .expect("insert should succeed");

        assert_eq!(grant.user_id, user);
        assert_eq!(grant.role_id, role);
        assert!(grant.created_at.timestamp() > 0);
        assert!(grant.updated_at >= grant.created_at);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_role_grants WHERE user_id = $1 AND role_id = $2",
        )
        .bind(user)
        .bind(role)
        .fetch_one(&pool)
        .await
        .expect("count should run");
        assert_eq!(count, 1, "the returned grant must be persisted");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_a_duplicate_grant_pair(pool: PgPool) {
        let user = seed_user(&pool).await;
        let role = seed_role(&pool, "dup-assigned-role").await;

        let repo = PgUserRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let params = CreateUserRoleGrant {
            user_id: user,
            role_id: role,
        };
        repo.insert_user_role_grant(&params)
            .await
            .expect("first grant should succeed");

        let err = repo
            .insert_user_role_grant(&params)
            .await
            .expect_err("the composite primary key must reject the duplicate pair");
        assert_sql_state(err, "23505");

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_role_grants")
            .fetch_one(&pool)
            .await
            .expect("count should run");
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_unknown_user_or_role_ids(pool: PgPool) {
        let user = seed_user(&pool).await;
        let role = seed_role(&pool, "fk-check-role").await;

        let repo = PgUserRoleGrantRepository::new(Database::from_pool(pool));

        let err = repo
            .insert_user_role_grant(&CreateUserRoleGrant {
                user_id: Uuid::new_v4(),
                role_id: role,
            })
            .await
            .expect_err("an unknown user id must violate the users FK");
        assert_sql_state(err, "23503");

        let err = repo
            .insert_user_role_grant(&CreateUserRoleGrant {
                user_id: user,
                role_id: Uuid::new_v4(),
            })
            .await
            .expect_err("an unknown role id must violate the roles FK");
        assert_sql_state(err, "23503");
    }
}
