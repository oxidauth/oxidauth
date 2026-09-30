use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    dev_prelude::BoxedError,
    role_permission_grants::RolePermission,
    roles::Role,
    user_permission_grants::UserPermission,
    users::User,
};

#[async_trait]
pub trait PermissionTreeServiceTrait: Send + Sync + 'static {
    async fn permission_tree(
        &self,
        params: &PermissionSearch,
    ) -> Result<PermissionsResponse, BoxedError>;
}

pub type PermissionTreeService = Arc<dyn PermissionTreeServiceTrait>;

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
        let (mut seen, mut out) = (HashSet::new(), Vec::new());
        self.collect(&mut seen, &mut out);
        out
    }

    // Nested-first walk with a shared seen-set: a permission reached through
    // several grant paths (role paths, or a role path plus a direct grant)
    // contributes exactly one entry — the first occurrence — and the flat
    // entitlement list stays in nested-before-direct order, only repeats are
    // removed. Dedup keys on the permission id, not the string: that is
    // exactly equivalent while `permissions` carries
    // `CONSTRAINT unique_grant_parts UNIQUE(realm, resource, action)`
    // (migration 20221019222410_create_permissions.sql), which makes
    // id ↔ `realm:resource:action` 1:1 — revisit if that ever changes.
    fn collect(&self, seen: &mut HashSet<Uuid>, out: &mut Vec<String>) {
        for child in &self.roles {
            child.collect(seen, out);
        }

        for grant in &self.permissions {
            if seen.insert(grant.permission.id) {
                out.push(grant.permission.to_string());
            }
        }
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
        let (mut seen, mut out) = (HashSet::new(), Vec::new());
        self.collect(&mut seen, &mut out);
        out
    }

    // Same first-occurrence/stable dedup as `UserNode::collect`; see the
    // id-keying invariant noted there.
    fn collect(&self, seen: &mut HashSet<Uuid>, out: &mut Vec<String>) {
        for child in &self.roles {
            child.collect(seen, out);
        }

        for grant in &self.permissions {
            if seen.insert(grant.permission.id) {
                out.push(grant.permission.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        permissions::Permission,
        role_permission_grants::RolePermissionGrant,
        user_permission_grants::UserPermissionGrant,
        users::{UserKind, UserStatus},
    };

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn permission(realm: &str, resource: &str, action: &str) -> Permission {
        Permission {
            id: Uuid::new_v4(),
            realm: realm.to_owned(),
            resource: resource.to_owned(),
            action: action.to_owned(),
            created_at: now(),
            updated_at: now(),
        }
    }

    fn user() -> User {
        User {
            id: Uuid::new_v4(),
            kind: UserKind::Human,
            status: UserStatus::Enabled,
            username: "tree-test".to_owned(),
            email: None,
            first_name: None,
            last_name: None,
            profile: Value::Null,
            created_at: now(),
            updated_at: now(),
        }
    }

    fn role(name: &str) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            created_at: now(),
            updated_at: now(),
        }
    }

    fn user_permission(user: &User, perm: Permission) -> UserPermission {
        UserPermission {
            permission: perm.clone(),
            grant: UserPermissionGrant {
                user_id: user.id,
                permission_id: perm.id,
                created_at: now(),
                updated_at: now(),
            },
        }
    }

    fn role_permission(role: &Role, perm: Permission) -> RolePermission {
        RolePermission {
            permission: perm.clone(),
            grant: RolePermissionGrant {
                role_id: role.id,
                permission_id: perm.id,
                created_at: now(),
                updated_at: now(),
            },
        }
    }

    // user --direct--> users:read
    //  \-- role "ops" --direct--> roles:read
    //              \-- role "admin" --direct--> oxidauth:**:**
    fn fixture() -> (UserNode, Vec<String>) {
        let user = user();

        let admin_role = role("admin");
        let admin_node = RoleNode {
            permissions: vec![role_permission(
                &admin_role,
                permission("oxidauth", "**", "**"),
            )],
            role: admin_role,
            roles: vec![],
        };

        let ops_role = role("ops");
        let ops_node = RoleNode {
            permissions: vec![role_permission(
                &ops_role,
                permission("oxidauth", "roles", "read"),
            )],
            role: ops_role,
            roles: vec![admin_node],
        };

        let node = UserNode {
            permissions: vec![user_permission(
                &user,
                permission("oxidauth", "users", "read"),
            )],
            roles: vec![ops_node],
            user,
        };

        let expected = vec![
            "oxidauth:**:**".to_string(),
            "oxidauth:roles:read".to_string(),
            "oxidauth:users:read".to_string(),
        ];

        (node, expected)
    }

    #[test]
    fn user_tree_flattens_nested_roles_before_direct_grants() {
        let (node, expected) = fixture();

        assert_eq!(node.permissions(), expected);

        // the enum delegates to the same traversal
        assert_eq!(PermissionTree::User(node).permissions(), expected);
    }

    #[test]
    fn role_tree_flattens_nested_grants() {
        let leaf_role = role("leaf");
        let leaf = RoleNode {
            permissions: vec![role_permission(
                &leaf_role,
                permission("oxidauth", "leaf", "read"),
            )],
            role: leaf_role,
            roles: vec![],
        };

        let parent_role = role("parent");
        let parent = RoleNode {
            permissions: vec![role_permission(
                &parent_role,
                permission("oxidauth", "parent", "read"),
            )],
            role: parent_role,
            roles: vec![leaf],
        };

        // nested first, then this node's direct grants, in declaration order
        assert_eq!(
            parent.permissions(),
            vec![
                "oxidauth:leaf:read".to_string(),
                "oxidauth:parent:read".to_string(),
            ]
        );

        assert_eq!(
            PermissionTree::Role(parent).permissions(),
            vec![
                "oxidauth:leaf:read".to_string(),
                "oxidauth:parent:read".to_string(),
            ]
        );
    }

    #[test]
    fn empty_nodes_yield_no_permissions() {
        let user = user();

        let empty_user = UserNode {
            user,
            roles: vec![],
            permissions: vec![],
        };

        assert!(
            empty_user
                .permissions()
                .is_empty()
        );

        let empty_role = role("empty");
        let empty_role = RoleNode {
            permissions: vec![],
            roles: vec![],
            role: empty_role,
        };

        assert!(
            empty_role
                .permissions()
                .is_empty()
        );
    }

    #[test]
    fn permissions_are_deduplicated_first_occurrence_wins() {
        // A permission granted both directly and through a role appears once in
        // the flattened entitlement list: the kernel flatten dedupes by
        // permission id, first occurrence wins, nested-before-direct order is
        // preserved.
        let user = user();
        let shared = permission("oxidauth", "shared", "read");

        let role_dto = role("dupes");
        let node = RoleNode {
            permissions: vec![role_permission(&role_dto, shared.clone())],
            role: role_dto,
            roles: vec![],
        };

        let user_node = UserNode {
            permissions: vec![user_permission(&user, shared)],
            roles: vec![node],
            user,
        };

        assert_eq!(
            user_node.permissions(),
            vec!["oxidauth:shared:read".to_string()]
        );
    }

    #[test]
    fn diamond_grants_collapse_to_first_occurrence_order() {
        // user -> roles [a, b]; a and b each walk their own copy of the same
        // leaf grant (no user-direct grants). The tree keeps one RoleNode per
        // path; the flat list keeps exactly one entry per distinct permission,
        // in first-occurrence (nested-first) order.
        let diamond = permission("oxidauth", "diamond", "read");

        let leaf_role = role("diamond_leaf");
        let leaf = RoleNode {
            permissions: vec![role_permission(&leaf_role, diamond.clone())],
            role: leaf_role,
            roles: vec![],
        };

        let leaf_copy_role = role("diamond_leaf");
        let leaf_copy = RoleNode {
            permissions: vec![role_permission(&leaf_copy_role, diamond)],
            role: leaf_copy_role,
            roles: vec![],
        };

        let a_role = role("diamond_a");
        let a = RoleNode {
            permissions: vec![role_permission(
                &a_role,
                permission("oxidauth", "a", "read"),
            )],
            role: a_role,
            roles: vec![leaf],
        };

        let b_role = role("diamond_b");
        let b = RoleNode {
            permissions: vec![role_permission(
                &b_role,
                permission("oxidauth", "b", "read"),
            )],
            role: b_role,
            roles: vec![leaf_copy],
        };

        let diamond_owner = user();
        let diamond_user = UserNode {
            user: diamond_owner,
            roles: vec![a, b],
            permissions: vec![],
        };

        let flat = diamond_user.permissions();

        assert_eq!(
            flat,
            vec![
                "oxidauth:diamond:read".to_string(),
                "oxidauth:a:read".to_string(),
                "oxidauth:b:read".to_string(),
            ]
        );
        assert_eq!(
            flat.iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            flat.len(),
            "set size equals list length: one entry per distinct permission"
        );
    }

    #[test]
    fn permissions_response_serde_shape() {
        let (node, expected) = fixture();

        let permissions = node.permissions();

        assert_eq!(permissions, expected);

        let response = PermissionsResponse {
            tree: PermissionTree::User(node),
            permissions,
        };

        let json = serde_json::to_value(&response).unwrap();

        // externally-tagged enum: {"User": {...}} with the three UserNode fields
        assert!(
            json["tree"]
                .get("User")
                .is_some()
        );
        assert!(
            json["tree"]
                .get("Role")
                .is_none()
        );

        let user_node = &json["tree"]["User"];

        assert_eq!(user_node["user"]["username"], json!("tree-test"));
        assert_eq!(user_node["user"]["kind"], json!("human"));
        assert_eq!(user_node["user"]["status"], json!("enabled"));
        assert_eq!(
            user_node["roles"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            user_node["permissions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );

        // grant DTOs ride the wire as {"permission": {...}, "grant": {...}}
        let direct = &user_node["permissions"][0];
        assert_eq!(direct["permission"]["realm"], json!("oxidauth"));
        assert_eq!(direct["permission"]["resource"], json!("users"));
        assert_eq!(direct["permission"]["action"], json!("read"));
        assert!(
            direct["grant"]
                .get("user_id")
                .is_some()
        );
        assert!(
            direct["grant"]
                .get("permission_id")
                .is_some()
        );

        // RoleNode nests recursively under "roles"
        let ops = &user_node["roles"][0];
        assert_eq!(ops["role"]["name"], json!("ops"));
        assert_eq!(ops["roles"][0]["role"]["name"], json!("admin"));

        assert_eq!(
            json["permissions"],
            json!([
                "oxidauth:**:**",
                "oxidauth:roles:read",
                "oxidauth:users:read",
            ])
        );
    }

    #[test]
    fn role_tree_serializes_under_the_role_tag() {
        let leaf_role = role("solo");
        let leaf = RoleNode {
            permissions: vec![role_permission(
                &leaf_role,
                permission("oxidauth", "solo", "read"),
            )],
            role: leaf_role,
            roles: vec![],
        };

        let permissions = leaf.permissions();

        let json = serde_json::to_value(PermissionsResponse {
            tree: PermissionTree::Role(leaf),
            permissions,
        })
        .unwrap();

        assert!(
            json["tree"]
                .get("Role")
                .is_some()
        );
        assert!(
            json["tree"]
                .get("User")
                .is_none()
        );
        assert_eq!(json["tree"]["Role"]["role"]["name"], json!("solo"));
        assert_eq!(json["permissions"], json!(["oxidauth:solo:read"]));
    }
}
