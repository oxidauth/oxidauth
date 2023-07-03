use async_recursion::async_recursion;
use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use self::{
    all::UserRow,
    permissions::all::{user_permissions_by_user_id, UserPermission},
    roles::all::user_roles_by_user_id,
};

use super::roles::{
    all::RoleRow,
    by_id::role_by_id,
    permissions::all::{permissions_by_role_id, RolePermission},
    roles::all::roles_by_role_id,
};

pub mod all;
pub mod authorities;
pub mod by_id;
pub mod by_username;
pub mod create;
pub mod delete;
pub mod permissions;
pub mod roles;
pub mod update;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", get(all::handler))
        .route("/", post(create::handler))
        .route("/:user_id", get(by_id::handler))
        .route("/:user_id", put(update::handler))
        .route("/:user_id", delete(delete::handler))
        .route("/by_username/:username", get(by_username::handler))
        .nest("/:user_id/authorities", authorities::router(database))
        .nest("/:user_id/permissions", permissions::router(database))
        .nest("/:user_id/roles", roles::router(database))
        .layer(Extension(database.clone()))
}

pub async fn permissions_as_tree(
    db: &mut PgConnection,
    source_id: PermissionSourceID,
) -> Result<PermissionsResult, sqlx::Error> {
    let result = match source_id {
        PermissionSourceID::User(user_id) => {
            let user = user_permissions_as_tree(db, user_id).await?;
            let permissions = user.permissions();

            PermissionsResult {
                tree: PermissionTree::User(user),
                permissions,
            }
        }

        PermissionSourceID::Role(role_id) => {
            let role = role_permissions_as_tree(db, role_id).await?;
            let permissions = role.permissions();

            PermissionsResult {
                tree: PermissionTree::Role(role),
                permissions,
            }
        }
    };

    Ok(result)
}

#[derive(Serialize, Deserialize)]
pub struct PermissionsResult {
    pub tree: PermissionTree,
    pub permissions: Vec<String>,
}

pub async fn user_permissions_as_tree(
    db: &mut PgConnection,
    user_id: Uuid,
) -> Result<UserNode, sqlx::Error> {
    let user = by_id::user_by_id(db, user_id).await?;

    let role_rows = user_roles_by_user_id(db, user.id).await?;

    let mut roles = Vec::new();

    for role in role_rows.into_iter() {
        let role = role_permissions_as_tree(db, role.role.id).await?;

        roles.push(role);
    }

    let permissions = user_permissions_by_user_id(db, user.id).await?;

    Ok(UserNode {
        user,
        roles,
        permissions,
    })
}

#[async_recursion]
pub async fn role_permissions_as_tree(
    db: &mut PgConnection,
    role_id: Uuid,
) -> Result<RoleNode, sqlx::Error> {
    let role = role_by_id(db, role_id).await?;

    let role_rows = roles_by_role_id(db, role_id).await?;

    let mut roles = Vec::new();

    for role in role_rows.into_iter() {
        let role = role_permissions_as_tree(db, role.role.id).await?;

        roles.push(role);
    }

    let permissions = permissions_by_role_id(db, role_id).await?;

    Ok(RoleNode {
        role,
        roles,
        permissions,
    })
}

pub enum PermissionSourceID {
    User(Uuid),
    Role(Uuid),
}

#[derive(Serialize, Deserialize)]
pub enum PermissionTree {
    User(UserNode),
    Role(RoleNode),
}

impl PermissionTree {
    pub fn permissions(&self) -> Vec<String> {
        match self {
            Self::User(user) => user.permissions(),
            Self::Role(role) => role.permissions(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserNode {
    pub user: UserRow,
    pub roles: Vec<RoleNode>,
    pub permissions: Vec<UserPermission>,
}

impl UserNode {
    pub fn permissions(&self) -> Vec<String> {
        let direct_permissions = self
            .permissions
            .iter()
            .map(|rp| rp.permission.to_string())
            .collect::<Vec<String>>();

        let permissions = self
            .roles
            .iter()
            .map(|rr| rr.permissions())
            .flatten()
            .collect::<Vec<String>>();

        [permissions, direct_permissions]
            .into_iter()
            .flatten()
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct RoleNode {
    pub role: RoleRow,
    pub roles: Vec<RoleNode>,
    pub permissions: Vec<RolePermission>,
}

impl RoleNode {
    pub fn permissions(&self) -> Vec<String> {
        let direct_permissions = self
            .permissions
            .iter()
            .map(|rp| rp.permission.to_string())
            .collect::<Vec<String>>();

        let permissions = self
            .roles
            .iter()
            .map(|rr| rr.permissions())
            .flatten()
            .collect::<Vec<String>>();

        [permissions, direct_permissions]
            .into_iter()
            .flatten()
            .collect()
    }
}
