//! Grant repository for `user_permission_grants`. The surface is intentional and
//! minimal — insert (pair), select-by-grantee (`user_id`), delete-by-pair only.
//! There is no grantee-wide or other bulk delete, uniformly across all four grant
//! tables (`user_role_grants`, `role_permission_grants`, `role_role_grants`).
//!
//! Why: the composite `PRIMARY KEY(user_id, permission_id)`
//! (`migrations/20221020012656_create_user_permission_grants.sql:7`) makes the
//! pair the only grant identity — there is no grant-id and no grantor column —
//! and every user-wide grant removal the product performs today goes through the
//! schema's `ON DELETE CASCADE` foreign keys
//! (`20221020012656_create_user_permission_grants.sql:9-10`; silent-cascade
//! objection registered as OXA-000020).
//!
//! Explicit bulk revoke is deferred until a real consumer exists; the
//! pre-approved repo-layer recipe lives in ticket `OXA-000054` —
//! executed across all four tables at once, `fetch_all` to `Vec` with an empty
//! vec (not `RowNotFound`) for a grantee without rows, and never the
//! `fetch_one` shape of `refresh_tokens/delete_refresh_token_by_user_id`
//! (anti-pattern, OXA-000023).
use oxidauth_kernel::user_permission_grants::create_user_permission_grant::CreateUserPermissionGrant;

pub mod delete_user_permission_grant;
pub mod insert_user_permission_grant;
pub mod select_user_permission_grants_by_user_id;

use oxidauth_kernel::{
    permissions::Permission,
    user_permission_grants::{UserPermission, UserPermissionGrant},
};

use crate::{Database, prelude::*};

#[derive(Debug, Clone)]
pub struct PgUserPermissionGrantRepository {
    db: Database,
}

impl PgUserPermissionGrantRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgUserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PgUserPermissionGrant> for UserPermissionGrant {
    fn from(value: PgUserPermissionGrant) -> Self {
        Self {
            user_id: value.user_id,
            permission_id: value.permission_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgUserPermission {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub permission_created_at: DateTime<Utc>,
    pub permission_updated_at: DateTime<Utc>,
}

impl From<PgUserPermission> for UserPermission {
    fn from(value: PgUserPermission) -> Self {
        Self {
            permission: Permission {
                id: value.permission_id,
                realm: value.realm,
                resource: value.resource,
                action: value.action,
                created_at: value.permission_created_at,
                updated_at: value.permission_updated_at,
            },
            grant: UserPermissionGrant {
                user_id: value.user_id,
                permission_id: value.permission_id,
                created_at: value.created_at,
                updated_at: value.updated_at,
            },
        }
    }
}
