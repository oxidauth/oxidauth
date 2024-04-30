use async_recursion::async_recursion;
use async_trait::async_trait;
use oxidauth_kernel::service::Service;
use oxidauth_kernel::users::User;
use oxidauth_repository::auth::tree::*;
use sqlx::PgConnection;

use crate::role_permission_grants::select_role_permission_grants_by_role_id::select_role_permission_grants_by_role_id_query;
use crate::role_role_grants::select_role_role_grants_by_parent_id::select_role_role_grants_by_parent_id_query;
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
        select_role_role_grants_by_parent_id_query(db, role_id).await?;

    let mut roles = Vec::new();

    for role in role_rows.into_iter() {
        let role = role_permissions_as_tree(db, role.child_id).await?;

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
