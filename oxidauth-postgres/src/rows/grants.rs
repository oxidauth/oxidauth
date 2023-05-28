use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct UserPermissionGrantRow {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct UserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(sqlx::FromRow)]
pub struct UserRoleGrantRow {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct UserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(sqlx::FromRow)]
pub struct RolePermissionGrantRow {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct RolePermissionGrant {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}

#[derive(sqlx::FromRow)]
pub struct RoleRoleGrantRow {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct RoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
}
