use async_recursion::async_recursion;
use async_trait::async_trait;
use oxidauth_kernel::service::Service;
use oxidauth_kernel::users::User;
use oxidauth_repository::auth::tree::*;
use sqlx::PgConnection;

use crate::role_permission_grants::select_role_permission_grants_by_role_id::select_role_permission_grants_by_role_id_query;
use crate::role_role_grants::select_role_role_grants_by_child_id::select_role_role_grants_by_child_id_query;
use crate::roles::select_role_by_id::select_role_by_id_query;
use crate::user_permission_grants::select_user_permission_grants_by_user_id::select_user_permission_grants_by_user_id_query;
use crate::user_role_grants::select_user_role_grants_by_user_id::select_user_role_grants_by_user_id_query;
use crate::users::select_user_by_id_query::select_user_by_id_query;
use crate::Database;

#[async_trait]
impl<'a> Service<&'a PermissionSearch> for Database {
    type Response = PermissionsResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "permission_tree_query", skip(self))]
    async fn call(
        &self,
        params: &'a PermissionSearch,
    ) -> Result<Self::Response, Self::Error> {
        let mut conn = self.pool.acquire().await?;

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

    let role_rows =
        select_user_role_grants_by_user_id_query(db, user.id).await?;

    let mut roles = Vec::new();

    for role in role_rows.into_iter() {
        let role = role_permissions_as_tree(db, role.role_id).await?;

        roles.push(role);
    }

    let permissions =
        select_user_permission_grants_by_user_id_query(db, user.id)
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

    let role_rows =
        select_role_role_grants_by_child_id_query(db, role_id).await?;

    let mut roles = Vec::new();

    for role in role_rows.into_iter() {
        let role = role_permissions_as_tree(db, role.parent_id).await?;

        roles.push(role);
    }

    let permissions =
        select_role_permission_grants_by_role_id_query(db, role_id)
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
    use sqlx::Row;
    use uuid::Uuid;

    use crate::Database;

    async fn insert_permission(
        pool: &sqlx::PgPool,
        realm: &str,
        resource: &str,
        action: &str,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO permissions (id, realm, resource, action, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, NOW(), NOW()) RETURNING id",
        )
        .bind(id)
        .bind(realm)
        .bind(resource)
        .bind(action)
        .fetch_one(pool)
        .await
        .unwrap()
        .get(0)
    }

    async fn insert_role(pool: &sqlx::PgPool, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO roles (id, name, created_at, updated_at) \
             VALUES ($1, $2, NOW(), NOW()) RETURNING id",
        )
        .bind(id)
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap()
        .get(0)
    }

    async fn grant_permission_to_role(
        pool: &sqlx::PgPool,
        role_id: Uuid,
        permission_id: Uuid,
    ) {
        sqlx::query(
            "INSERT INTO role_permission_grants (role_id, permission_id, created_at, updated_at) \
             VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn grant_role_to_role(
        pool: &sqlx::PgPool,
        parent_id: Uuid,
        child_id: Uuid,
    ) {
        sqlx::query(
            "INSERT INTO role_role_grants (parent_id, child_id, created_at, updated_at) \
             VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(parent_id)
        .bind(child_id)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn fetch_permissions(
        pool: &sqlx::PgPool,
        search: PermissionSearch,
    ) -> Vec<String> {
        let db = Database { pool: pool.clone() };

        <Database as oxidauth_kernel::service::Service<&PermissionSearch>>::call(
                &db,
                &search,
            )
            .await
            .unwrap()
            .permissions
    }

    #[sqlx::test]
    async fn child_role_inherits_permission_from_parent_via_role_role_grant(
        pool: sqlx::PgPool,
    ) {
        let parent_id = insert_role(&pool, "firefly:parent").await;
        let child_id = insert_role(&pool, "firefly:child").await;

        grant_role_to_role(&pool, parent_id, child_id).await;

        let permission_id =
            insert_permission(&pool, "firefly", "serenity", "fly").await;
        grant_permission_to_role(&pool, parent_id, permission_id).await;

        let child_permissions =
            fetch_permissions(&pool, PermissionSearch::Role(child_id)).await;

        assert!(
            child_permissions.contains(&"firefly:serenity:fly".to_string()),
            "child role should inherit parent's permission via role_role_grant, got: {:?}",
            child_permissions,
        );
    }

    #[sqlx::test]
    async fn parent_role_does_not_inherit_permissions_from_child_via_role_role_grant(
        pool: sqlx::PgPool,
    ) {
        let parent_id = insert_role(&pool, "firefly:parent").await;
        let child_id = insert_role(&pool, "firefly:child").await;

        grant_role_to_role(&pool, parent_id, child_id).await;

        let permission_id =
            insert_permission(&pool, "firefly", "serenity", "fly").await;
        grant_permission_to_role(&pool, child_id, permission_id).await;

        let parent_permissions =
            fetch_permissions(&pool, PermissionSearch::Role(parent_id)).await;

        assert!(
            !parent_permissions.contains(&"firefly:serenity:fly".to_string()),
            "parent role should NOT inherit child's permission via role_role_grant, got: {:?}",
            parent_permissions,
        );
    }
}
