use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dev_prelude::{BoxedError, Service},
    role_permission_grants::RolePermission,
    roles::Role,
    user_permission_grants::UserPermission,
    users::User,
};

pub type PermissionTreeService = Arc<
    dyn for<'a> Service<
        &'a PermissionSearch,
        Response = PermissionsResponse,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
pub enum PermissionSearch {
    User(Uuid),
    Role(Uuid),
}

#[derive(Debug, Serialize)]
pub struct PermissionsResponse {
    pub tree: PermissionTree,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct UserNode {
    pub user: User,
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

#[derive(Debug, Serialize)]
pub struct RoleNode {
    pub role: Role,
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
