use async_recursion::async_recursion;
use oxidauth_kernel::users::User;
use oxidauth_repository::auth::tree::*;
use sqlx::PgConnection;

use super::*;
use crate::{
    role_permission_grants::select_role_permission_grants_by_role_id::select_role_permission_grants_by_role_id_query,
    role_role_grants::select_role_role_grants_by_parent_id::select_role_role_grants_by_parent_id_query,
    roles::select_role_by_id::select_role_by_id_query,
    user_permission_grants::select_user_permission_grants_by_user_id::select_user_permission_grants_by_user_id_query,
    user_role_grants::select_user_role_grants_by_user_id::select_user_role_grants_by_user_id_query,
    users::select_user_by_id_query::select_user_by_id_query,
};

#[async_trait]
impl PermissionTreeQuery for PgAuthRepository {
    // span name intentionally descriptive: module dir is `tree`; mechanical `tree_query` would be
    // worse
    #[tracing::instrument(name = "permission_tree_query", skip(self))]
    async fn permission_tree(
        &self,
        params: &PermissionSearch,
    ) -> Result<PermissionsResponse, BoxedError> {
        let pool = self.db.read_pool();
        let mut conn = pool.acquire().await?;

        permissions_as_tree(&mut conn, params).await
    }
}

async fn permissions_as_tree(
    db: &mut PgConnection,
    source_id: &PermissionSearch,
) -> Result<PermissionsResponse, BoxedError> {
    let result = match source_id {
        PermissionSearch::User(user_id) => {
            let user = user_permissions_as_tree(db, *user_id).await?;
            let permissions = user.permissions();

            PermissionsResponse {
                tree: PermissionTree::User(user),
                permissions,
            }
        },

        PermissionSearch::Role(role_id) => {
            let role = role_permissions_as_tree(db, *role_id).await?;
            let permissions = role.permissions();

            PermissionsResponse {
                tree: PermissionTree::Role(role),
                permissions,
            }
        },
    };

    Ok(result)
}

async fn user_permissions_as_tree(
    db: &mut PgConnection,
    user_id: Uuid,
) -> Result<UserNode, BoxedError> {
    let user: User = select_user_by_id_query(db, user_id)
        .await?
        .try_into()?;

    let role_rows = select_user_role_grants_by_user_id_query(db, user.id).await?;

    let mut roles = Vec::new();

    for role in role_rows.into_iter() {
        let role = role_permissions_as_tree(db, role.role_id).await?;

        roles.push(role);
    }

    let permissions = select_user_permission_grants_by_user_id_query(db, user.id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(UserNode {
        user,
        roles,
        permissions,
    })
}

#[async_recursion]
async fn role_permissions_as_tree(
    db: &mut PgConnection,
    role_id: Uuid,
) -> Result<RoleNode, BoxedError> {
    let role = select_role_by_id_query(db, role_id)
        .await?
        .into();

    let role_rows = select_role_role_grants_by_parent_id_query(db, role_id).await?;

    let mut roles = Vec::new();

    for role in role_rows.into_iter() {
        let role = role_permissions_as_tree(db, role.child_id).await?;

        roles.push(role);
    }

    let permissions = select_role_permission_grants_by_role_id_query(db, role_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(RoleNode {
        role,
        roles,
        permissions,
    })
}

#[cfg(test)]
mod tests {
    use oxidauth_kernel::auth::tree::PermissionSearch;
    use sqlx::PgPool;

    use super::*;
    use crate::{
        Database,
        auth::PgAuthRepository,
        test_fixtures::{seed_permission, seed_role, seed_user_named},
    };

    fn repo(pool: &PgPool) -> PgAuthRepository {
        PgAuthRepository::new(Database::from_pool(pool.clone()))
    }

    async fn exec(pool: &PgPool, sql: &str, first: Uuid, second: Uuid) {
        sqlx::query(sql)
            .bind(first)
            .bind(second)
            .execute(pool)
            .await
            .expect("seed grant");
    }

    struct Fixture {
        user: Uuid,
        parent: Uuid,
        child: Uuid,
        grandchild: Uuid,
    }

    /// user -> parent -> child -> grandchild, with `app:thing:read` granted at the
    /// user, the parent AND the child (three paths to the same permission).
    async fn seed_full_fixture(pool: &PgPool) -> Fixture {
        let user = seed_user_named(pool, "tree_user").await;
        let parent = seed_role(pool, "tree_parent").await;
        let child = seed_role(pool, "tree_child").await;
        let grandchild = seed_role(pool, "tree_grandchild").await;

        let read = seed_permission(pool, "app", "thing", "read").await;
        let write = seed_permission(pool, "app", "thing", "write").await;
        let del = seed_permission(pool, "app", "other", "delete").await;

        exec(
            pool,
            "INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)",
            user,
            parent,
        )
        .await;
        exec(
            pool,
            "INSERT INTO role_role_grants (parent_id, child_id) VALUES ($1, $2)",
            parent,
            child,
        )
        .await;
        exec(
            pool,
            "INSERT INTO role_role_grants (parent_id, child_id) VALUES ($1, $2)",
            child,
            grandchild,
        )
        .await;
        exec(
            pool,
            "INSERT INTO role_permission_grants (role_id, permission_id) VALUES ($1, $2)",
            parent,
            read,
        )
        .await;
        exec(
            pool,
            "INSERT INTO role_permission_grants (role_id, permission_id) VALUES ($1, $2)",
            child,
            read,
        )
        .await;
        exec(
            pool,
            "INSERT INTO role_permission_grants (role_id, permission_id) VALUES ($1, $2)",
            grandchild,
            write,
        )
        .await;
        exec(
            pool,
            "INSERT INTO user_permission_grants (user_id, permission_id) VALUES ($1, $2)",
            user,
            read,
        )
        .await;
        exec(
            pool,
            "INSERT INTO user_permission_grants (user_id, permission_id) VALUES ($1, $2)",
            user,
            del,
        )
        .await;

        Fixture {
            user,
            parent,
            child,
            grandchild,
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_assemble_user_tree_with_nested_role_grants(pool: PgPool) {
        let fixture = seed_full_fixture(&pool).await;

        let response = repo(&pool)
            .permission_tree(&PermissionSearch::User(fixture.user))
            .await
            .expect("tree query should succeed");

        let PermissionTree::User(node) = &response.tree else {
            panic!("expected a user permission tree");
        };

        assert_eq!(node.user.id, fixture.user);
        assert_eq!(node.roles.len(), 1, "one role granted to the user");

        let parent = &node.roles[0];
        assert_eq!(parent.role.name, "tree_parent");
        assert_eq!(parent.permissions.len(), 1);
        assert_eq!(
            parent.permissions[0]
                .permission
                .to_string(),
            "app:thing:read"
        );

        let child = &parent.roles[0];
        assert_eq!(child.role.name, "tree_child");
        assert_eq!(child.permissions.len(), 1, "nested role grants included");
        assert_eq!(child.roles.len(), 1);

        let grandchild = &child.roles[0];
        assert_eq!(grandchild.role.name, "tree_grandchild");
        assert!(grandchild.roles.is_empty());

        assert_eq!(node.permissions.len(), 2, "user-direct grants");

        // OXA-000039: `app:thing:read` is granted via three paths
        // (user-direct, parent role, nested child role); the flat entitlement
        // list dedupes by permission id, so it carries one entry per distinct
        // permission while the tree above still shows every grant path.
        let mut flattened = response.permissions.clone();
        flattened.sort();
        assert_eq!(
            flattened,
            vec![
                "app:other:delete".to_owned(),
                "app:thing:read".to_owned(),
                "app:thing:write".to_owned(),
            ]
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_assemble_role_tree_for_role_search(pool: PgPool) {
        let fixture = seed_full_fixture(&pool).await;

        let response = repo(&pool)
            .permission_tree(&PermissionSearch::Role(fixture.parent))
            .await
            .expect("tree query should succeed");

        let PermissionTree::Role(root) = &response.tree else {
            panic!("expected a role permission tree");
        };

        assert_eq!(root.role.name, "tree_parent");
        assert_eq!(root.roles.len(), 1);
        assert_eq!(root.roles[0].role.name, "tree_child");
        assert_eq!(
            root.roles[0].roles[0]
                .role
                .name,
            "tree_grandchild"
        );

        // the user-direct grant is NOT part of a role search; within the role
        // subtree `app:thing:read` is reachable twice (parent + nested child)
        // and the flat list now keeps a single entry (see dedup note in the
        // user-tree test)
        let mut flattened = response.permissions.clone();
        flattened.sort();
        assert_eq!(
            flattened,
            vec!["app:thing:read".to_owned(), "app:thing:write".to_owned(),]
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_an_empty_tree_for_a_user_without_grants(pool: PgPool) {
        let user = seed_user_named(&pool, "bare_user").await;

        let response = repo(&pool)
            .permission_tree(&PermissionSearch::User(user))
            .await
            .expect("tree query should succeed");

        let PermissionTree::User(node) = &response.tree else {
            panic!("expected a user permission tree");
        };

        assert_eq!(node.user.id, user);
        assert!(node.roles.is_empty());
        assert!(node.permissions.is_empty());
        assert!(
            response
                .permissions
                .is_empty()
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_error_for_an_unknown_user(pool: PgPool) {
        let err = repo(&pool)
            .permission_tree(&PermissionSearch::User(Uuid::new_v4()))
            .await
            .expect_err("unknown user must surface as an error");

        assert!(
            format!("{err:?}").contains("RowNotFound"),
            "expected RowNotFound, got: {err:?}"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_recurse_forever_on_a_role_role_grant_cycle(pool: PgPool) {
        // user -> role A; role_role_grants A->B AND B->A; A carries a permission
        let user = seed_user_named(&pool, "cycle_user").await;
        let a = seed_role(&pool, "cycle_a").await;
        let b = seed_role(&pool, "cycle_b").await;
        let perm = seed_permission(&pool, "cycle", "thing", "read").await;

        exec(
            &pool,
            "INSERT INTO user_role_grants (user_id, role_id) VALUES ($1, $2)",
            user,
            a,
        )
        .await;
        exec(
            &pool,
            "INSERT INTO role_role_grants (parent_id, child_id) VALUES ($1, $2)",
            a,
            b,
        )
        .await;
        exec(
            &pool,
            "INSERT INTO role_role_grants (parent_id, child_id) VALUES ($1, $2)",
            b,
            a,
        )
        .await;
        exec(
            &pool,
            "INSERT INTO role_permission_grants (role_id, permission_id) VALUES ($1, $2)",
            a,
            perm,
        )
        .await;

        // BUG(pinned): `role_permissions_as_tree` recurses through
        // `select_role_role_grants_by_parent_id` with no visited-set and no SQL
        // guard, while the insert side (see role_role_grants tests) accepts cyclic
        // grants — a cycle makes tree assembly recurse unboundedly.
        //
        // Observed behavior: on the default 2 MB test-thread stack the recursion
        // SIGABRTs with a stack overflow in well under a second (each poll walks
        // the whole nested-box future chain), killing the whole test binary. To
        // prove-and-pin the NON-termination without aborting the suite, the call
        // runs on a dedicated thread with a 1 GB stack so the 3 s
        // `tokio::time::timeout` wins first. Observed cleanup: the timeout drops
        // the recursion between DB round-trips, the connection is returned to the
        // pool, sqlx drops the test's database on success, and the suite
        // finishes in ~3 s instead of hanging.
        let outcome = std::thread::Builder::new()
            .stack_size(1024 * 1024 * 1024)
            .spawn({
                let pool = pool.clone();
                move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("probe runtime");

                    rt.block_on(async move {
                        let service = repo(&pool);
                        tokio::time::timeout(
                            std::time::Duration::from_secs(3),
                            service.permission_tree(&PermissionSearch::User(user)),
                        )
                        .await
                    })
                }
            })
            .expect("spawn cycle probe thread")
            .join()
            .expect("cycle probe thread panicked");

        assert!(
            outcome.is_err(),
            "pinned bug: tree assembly with a role cycle must not terminate within 3s; \
             it completed with {:?} entries instead",
            outcome.map(|r| r.map(|resp| resp.permissions.len()))
        );
    }
}
