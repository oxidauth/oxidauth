use oxidauth_repository::user_role_grants::select_user_role_grants_by_user_id::*;

use super::*;

#[async_trait]
impl SelectUserRoleGrantsByUserIdQuery for PgUserRoleGrantRepository {
    #[tracing::instrument(name = "select_user_role_grants_by_user_id_query", skip(self))]
    async fn select_user_role_grants_by_user_id(
        &self,
        params: &ListUserRoleGrantsByUserId,
    ) -> Result<Vec<UserRole>, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let user_role_grants = select_user_role_grants_by_user_id_query(&mut conn, params.user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(user_role_grants)
    }
}

pub async fn select_user_role_grants_by_user_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<PgUserRole>, BoxedError> {
    let user_role_grants = sqlx::query_as::<_, PgUserRole>(include_str!(
        "./select_user_role_grants_by_user_id_query.sql"
    ))
    .bind(user_id)
    .fetch_all(conn)
    .await?;

    Ok(user_role_grants)
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{Database, test_fixtures::seed_user_named};

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_an_empty_list_for_a_user_without_roles(pool: PgPool) {
        let user = seed_user_named(&pool, "roleless-user").await;

        let repo = PgUserRoleGrantRepository::new(Database::from_pool(pool));
        let grants = repo
            .select_user_role_grants_by_user_id(&ListUserRoleGrantsByUserId { user_id: user })
            .await
            .expect("an empty result is not an error");

        assert!(grants.is_empty(), "no grants seeded, none should come back");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_join_the_role_and_pin_the_grant_timestamp_mapping(pool: PgPool) {
        let user = seed_user_named(&pool, "multi-role-user").await;
        let other = seed_user_named(&pool, "other-role-user").await;

        // backdate the role row so its own timestamps are distinguishable
        let role = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO roles (id, name, created_at, updated_at)
             VALUES ($1, $2, '2000-01-01T00:00:00Z', '2000-01-01T00:00:00Z')",
        )
        .bind(role)
        .bind("backdated-role")
        .execute(&pool)
        .await
        .expect("seed backdated role");

        let other_role = Uuid::new_v4();
        sqlx::query("INSERT INTO roles (id, name) VALUES ($1, $2)")
            .bind(other_role)
            .bind("foreign-role")
            .execute(&pool)
            .await
            .expect("seed foreign role");

        sqlx::query("INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)")
            .bind(user)
            .bind(role)
            .execute(&pool)
            .await
            .expect("seed grant");
        sqlx::query("INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)")
            .bind(other)
            .bind(other_role)
            .execute(&pool)
            .await
            .expect("seed foreign grant");

        let repo = PgUserRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let grants = repo
            .select_user_role_grants_by_user_id(&ListUserRoleGrantsByUserId { user_id: user })
            .await
            .expect("select should succeed");

        assert_eq!(grants.len(), 1, "only this user's grant");
        let granted = &grants[0];
        assert_eq!(granted.grant.user_id, user);
        assert_eq!(granted.grant.role_id, role);
        assert_eq!(granted.role.id, role);
        assert_eq!(granted.role.name, "backdated-role");

        let role_ts: DateTime<Utc> =
            sqlx::query_scalar("SELECT created_at FROM roles WHERE id = $1")
                .bind(role)
                .fetch_one(&pool)
                .await
                .expect("role timestamp should be readable");
        assert!(
            role_ts.timestamp() < 1_000_000_000,
            "the seeded role row is backdated to 2000"
        );

        // BUG(pinned): the From<PgUserRole> impl maps the GRANT's created_at /
        // updated_at into UserRole.role.created_at / role.updated_at — the SQL's
        // role_created_at / role_updated_at columns are selected but ignored —
        // so the returned role carries the grant's timestamps, not the role's.
        assert_eq!(granted.role.created_at, granted.grant.created_at);
        assert_ne!(granted.role.created_at, role_ts);
    }
}
