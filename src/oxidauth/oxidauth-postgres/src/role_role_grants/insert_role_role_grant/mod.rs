use oxidauth_repository::role_role_grants::insert_role_role_grant::*;

use super::*;

#[async_trait]
impl InsertRoleRoleGrantQuery for PgRoleRoleGrantRepository {
    #[tracing::instrument(name = "insert_role_role_grant_query", skip(self))]
    async fn insert_role_role_grant(
        &self,
        params: &CreateRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError> {
        let result =
            sqlx::query_as::<_, PgRoleRoleGrant>(include_str!("./insert_role_role_grant.sql"))
                .bind(params.parent_id)
                .bind(params.child_id)
                .fetch_one(&self.db.write_pool())
                .await?;

        let role_role_grant = result.into();

        Ok(role_role_grant)
    }
}

#[cfg(test)]
mod tests {

    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        test_fixtures::{assert_sql_state, seed_role},
    };

    async fn count_edges(pool: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM role_role_grants")
            .fetch_one(pool)
            .await
            .expect("count should run")
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_grant_a_child_role_to_a_parent_role(pool: PgPool) {
        let parent = seed_role(&pool, "parent-role").await;
        let child = seed_role(&pool, "child-role").await;

        let repo = PgRoleRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let grant = repo
            .insert_role_role_grant(&CreateRoleRoleGrant {
                parent_id: parent,
                child_id: child,
            })
            .await
            .expect("insert should succeed");

        assert_eq!(grant.parent_id, parent);
        assert_eq!(grant.child_id, child);
        assert!(grant.created_at.timestamp() > 0);
        assert!(grant.updated_at >= grant.created_at);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM role_role_grants WHERE parent_id = $1 AND child_id = $2",
        )
        .bind(parent)
        .bind(child)
        .fetch_one(&pool)
        .await
        .expect("count should run");
        assert_eq!(count, 1, "the returned edge must be persisted");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_allow_a_self_referencing_grant_parent_eq_child(pool: PgPool) {
        // BUG(pinned): neither the schema (no parent <> child CHECK) nor the
        // repository guards self-reference: a role can be granted to itself and
        // the row is stored. This table feeds the recursive permission-tree
        // walk in auth/tree, so a self-loop is possible input for that query.
        let role = seed_role(&pool, "self-role").await;

        let repo = PgRoleRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let grant = repo
            .insert_role_role_grant(&CreateRoleRoleGrant {
                parent_id: role,
                child_id: role,
            })
            .await
            .expect("pinned current behavior: parent == child is accepted");

        assert_eq!(grant.parent_id, grant.child_id);
        assert_eq!(count_edges(&pool).await, 1);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_allow_a_two_role_cycle(pool: PgPool) {
        // BUG(pinned): cycles are not detected anywhere — A->B followed by
        // B->A both succeed (the composite PK only rejects the identical
        // (parent, child) pair), so the recursive tree in auth/tree can be
        // fed a cyclic graph.
        let a = seed_role(&pool, "cycle-a").await;
        let b = seed_role(&pool, "cycle-b").await;

        let repo = PgRoleRoleGrantRepository::new(Database::from_pool(pool.clone()));
        repo.insert_role_role_grant(&CreateRoleRoleGrant {
            parent_id: a,
            child_id: b,
        })
        .await
        .expect("pinned current behavior: A -> B succeeds");
        repo.insert_role_role_grant(&CreateRoleRoleGrant {
            parent_id: b,
            child_id: a,
        })
        .await
        .expect("pinned current behavior: the reverse edge B -> A also succeeds");

        assert_eq!(
            count_edges(&pool).await,
            2,
            "both directions of the cycle exist"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_reject_duplicate_pairs_and_unknown_roles(pool: PgPool) {
        let parent = seed_role(&pool, "dup-parent").await;
        let child = seed_role(&pool, "dup-child").await;

        let repo = PgRoleRoleGrantRepository::new(Database::from_pool(pool.clone()));
        let params = CreateRoleRoleGrant {
            parent_id: parent,
            child_id: child,
        };
        repo.insert_role_role_grant(&params)
            .await
            .expect("first edge should succeed");

        let err = repo
            .insert_role_role_grant(&params)
            .await
            .expect_err("the composite primary key must reject the duplicate pair");
        assert_sql_state(err, "23505");

        let err = repo
            .insert_role_role_grant(&CreateRoleRoleGrant {
                parent_id: Uuid::new_v4(),
                child_id: child,
            })
            .await
            .expect_err("an unknown parent must violate the roles FK");
        assert_sql_state(err, "23503");

        let err = repo
            .insert_role_role_grant(&CreateRoleRoleGrant {
                parent_id: parent,
                child_id: Uuid::new_v4(),
            })
            .await
            .expect_err("an unknown child must violate the roles FK");
        assert_sql_state(err, "23503");

        assert_eq!(count_edges(&pool).await, 1, "only the first edge exists");
    }
}
