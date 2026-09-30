use oxidauth_repository::user_permission_grants::select_user_permission_grants_by_user_id::*;

use super::*;

#[async_trait]
impl SelectUserPermissionGrantsByUserIdQuery for PgUserPermissionGrantRepository {
    #[tracing::instrument(name = "select_user_permission_grants_by_user_id_query", skip(self))]
    async fn select_user_permission_grants_by_user_id(
        &self,
        params: &ListUserPermissionGrantsByUserId,
    ) -> Result<Vec<UserPermission>, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let user_permission_grants =
            select_user_permission_grants_by_user_id_query(&mut conn, params.user_id)
                .await?
                .into_iter()
                .map(Into::into)
                .collect();

        Ok(user_permission_grants)
    }
}

pub async fn select_user_permission_grants_by_user_id_query(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<PgUserPermission>, BoxedError> {
    let user_permission_grants = sqlx::query_as::<_, PgUserPermission>(include_str!(
        "./select_user_permission_grants_by_user_id_query.sql"
    ))
    .bind(user_id)
    .fetch_all(conn)
    .await?;

    Ok(user_permission_grants)
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_permission, seed_user_named, seed_user_permission_grant},
    };

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_an_empty_list_for_a_user_without_grants(pool: PgPool) {
        let user = seed_user_named(&pool, "barren-user").await;

        let repo = PgUserPermissionGrantRepository::new(Database::from_pool(pool));
        let grants = repo
            .select_user_permission_grants_by_user_id(&ListUserPermissionGrantsByUserId {
                user_id: user,
            })
            .await
            .expect("an empty result is not an error");

        assert!(grants.is_empty(), "no grants seeded, none should come back");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_only_this_users_grants_with_joined_permission(pool: PgPool) {
        let user = seed_user_named(&pool, "granted-user").await;
        let other = seed_user_named(&pool, "other-user").await;
        let read = seed_permission(&pool, "scoped", "profile", "read").await;
        let write = seed_permission(&pool, "scoped", "profile", "write").await;
        let foreign = seed_permission(&pool, "foreign", "profile", "read").await;
        seed_user_permission_grant(&pool, user, read).await;
        seed_user_permission_grant(&pool, user, write).await;
        seed_user_permission_grant(&pool, other, foreign).await;

        let repo = PgUserPermissionGrantRepository::new(Database::from_pool(pool));
        let grants = repo
            .select_user_permission_grants_by_user_id(&ListUserPermissionGrantsByUserId {
                user_id: user,
            })
            .await
            .expect("select should succeed");

        assert_eq!(grants.len(), 2, "only the two grants for this user");
        let mut parts: Vec<String> = grants
            .iter()
            .map(|g| {
                assert_eq!(g.grant.user_id, user);
                assert_eq!(g.grant.permission_id, g.permission.id);
                format!(
                    "{}:{}:{}",
                    g.permission.realm, g.permission.resource, g.permission.action
                )
            })
            .collect();
        parts.sort();
        assert_eq!(
            parts,
            vec![
                "scoped:profile:read".to_owned(),
                "scoped:profile:write".to_owned()
            ],
            "grants of other users must not leak into the result"
        );

        let ids: Vec<uuid::Uuid> = grants
            .iter()
            .map(|g| g.permission.id)
            .collect();
        assert!(ids.contains(&read) && ids.contains(&write) && !ids.contains(&foreign));
    }
}
