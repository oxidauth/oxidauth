use oxidauth_kernel::role_permission_grants::list_role_permission_grants_by_role_id::ListRolePermissionGrantsByRoleId;
use oxidauth_repository::role_permission_grants::select_role_permission_grants_by_role_id::*;

use super::*;

#[async_trait]
impl SelectRolePermissionGrantsByRoleIdQuery for PgRolePermissionGrantRepository {
    #[tracing::instrument(name = "select_role_permission_grants_by_role_id_query", skip(self))]
    async fn select_role_permission_grants_by_role_id(
        &self,
        params: &ListRolePermissionGrantsByRoleId,
    ) -> Result<Vec<RolePermission>, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let result =
            select_role_permission_grants_by_role_id_query(&mut conn, params.role_id).await?;

        let role_role_grant = result
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(role_role_grant)
    }
}

pub async fn select_role_permission_grants_by_role_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<PgRolePermission>, BoxedError> {
    let result = sqlx::query_as::<_, PgRolePermission>(include_str!(
        "./select_role_permission_grants_by_role_id.sql"
    ))
    .bind(role_id)
    .fetch_all(conn)
    .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{seed_permission, seed_role, seed_role_permission_grant},
    };

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_an_empty_list_for_a_role_without_grants(pool: PgPool) {
        let role = seed_role(&pool, "barren-role").await;

        let repo = PgRolePermissionGrantRepository::new(Database::from_pool(pool));
        let grants = repo
            .select_role_permission_grants_by_role_id(&ListRolePermissionGrantsByRoleId {
                role_id: role,
            })
            .await
            .expect("an empty result is not an error");

        assert!(grants.is_empty(), "no grants seeded, none should come back");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_only_this_roles_grants_with_joined_permission_parts(pool: PgPool) {
        let role = seed_role(&pool, "granted-role").await;
        let other = seed_role(&pool, "other-role").await;
        let read = seed_permission(&pool, "scoped", "reports", "read").await;
        let write = seed_permission(&pool, "scoped", "reports", "write").await;
        let foreign = seed_permission(&pool, "foreign", "reports", "read").await;
        seed_role_permission_grant(&pool, role, read).await;
        seed_role_permission_grant(&pool, role, write).await;
        seed_role_permission_grant(&pool, other, foreign).await;

        let repo = PgRolePermissionGrantRepository::new(Database::from_pool(pool));
        let grants = repo
            .select_role_permission_grants_by_role_id(&ListRolePermissionGrantsByRoleId {
                role_id: role,
            })
            .await
            .expect("select should succeed");

        assert_eq!(grants.len(), 2, "only the two grants for this role");
        let mut parts: Vec<String> = grants
            .iter()
            .map(|g| {
                assert_eq!(g.grant.role_id, role);
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
                "scoped:reports:read".to_owned(),
                "scoped:reports:write".to_owned()
            ],
            "grants of other roles must not leak into the result"
        );

        let ids: Vec<uuid::Uuid> = grants
            .iter()
            .map(|g| g.permission.id)
            .collect();
        assert!(ids.contains(&read) && ids.contains(&write) && !ids.contains(&foreign));
    }
}
