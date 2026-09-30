use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::ListRoleRoleGrantsByParentId;
use oxidauth_repository::role_role_grants::select_role_role_grants_by_parent_id::*;

use super::*;

#[async_trait]
impl SelectRoleRoleGrantsByParentIdQuery for PgRoleRoleGrantRepository {
    #[tracing::instrument(name = "select_role_role_grants_by_parent_id_query", skip(self))]
    async fn select_role_role_grants_by_parent_id(
        &self,
        params: &ListRoleRoleGrantsByParentId,
    ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        let result =
            select_role_role_grants_by_parent_id_query(&mut conn, params.parent_id).await?;

        let role_role_grant = result
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(role_role_grant)
    }
}

pub async fn select_role_role_grants_by_parent_id_query(
    conn: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<PgRoleRoleGrantDetail>, BoxedError> {
    let result = sqlx::query_as::<_, PgRoleRoleGrantDetail>(include_str!(
        "./select_role_role_grants_by_parent_id.sql"
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
        test_fixtures::{seed_role, seed_role_role_grant},
    };

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_an_empty_list_for_a_parent_without_children(pool: PgPool) {
        let parent = seed_role(&pool, "childless-parent").await;

        let repo = PgRoleRoleGrantRepository::new(Database::from_pool(pool));
        let edges = repo
            .select_role_role_grants_by_parent_id(&ListRoleRoleGrantsByParentId {
                parent_id: parent,
            })
            .await
            .expect("an empty result is not an error");

        assert!(edges.is_empty(), "no edges seeded, none should come back");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_only_this_parents_children_with_joined_role(pool: PgPool) {
        let parent = seed_role(&pool, "my-parent").await;
        let first = seed_role(&pool, "first-child").await;
        let second = seed_role(&pool, "second-child").await;
        let stranger_parent = seed_role(&pool, "stranger-parent").await;
        seed_role_role_grant(&pool, parent, first).await;
        seed_role_role_grant(&pool, parent, second).await;
        seed_role_role_grant(&pool, stranger_parent, first).await;

        let repo = PgRoleRoleGrantRepository::new(Database::from_pool(pool));
        let edges = repo
            .select_role_role_grants_by_parent_id(&ListRoleRoleGrantsByParentId {
                parent_id: parent,
            })
            .await
            .expect("select should succeed");

        assert_eq!(edges.len(), 2, "only this parent's two edges");
        let mut children: Vec<(uuid::Uuid, String)> = edges
            .iter()
            .map(|e| {
                assert_eq!(e.grant.parent_id, parent);
                assert_eq!(e.grant.child_id, e.role.id, "detail joins the child role");
                (e.role.id, e.role.name.clone())
            })
            .collect();
        children.sort();
        let mut expected = vec![
            (first, "first-child".to_owned()),
            (second, "second-child".to_owned()),
        ];
        expected.sort();
        assert_eq!(
            children, expected,
            "the joined child id and name must round-trip"
        );
    }
}
