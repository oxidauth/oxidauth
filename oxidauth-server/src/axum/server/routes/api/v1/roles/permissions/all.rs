use axum::{extract::Path, response::IntoResponse, Extension};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::{server::routes::api::v1::permissions::all::PermissionRow, Response};

#[derive(Deserialize)]
pub struct RolePermissionsAllReq {
    pub role_id: Uuid,
}

#[derive(Serialize)]
pub struct RolePermissionsAllRes {
    pub permissions: Vec<RolePermission>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<RolePermissionsAllReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = permissions_by_role_id(&mut db, params.role_id)
        .await
        .map(|permissions| RolePermissionsAllRes { permissions })
        .map_err(|error| error.to_string());

    match result {
        Ok(payload) => Response::success(payload).json(),
        Err(errors) => Response::fail(errors).json(),
    }
}

pub const QUERY: &str = r#"
    SELECT
        role_permission_grants.role_id,
        role_permission_grants.permission_id,
        role_permission_grants.created_at,
        role_permission_grants.updated_at,
        permissions.realm,
        permissions.resource,
        permissions.action,
        permissions.created_at AS permission_created_at,
        permissions.updated_at AS permission_updated_at
    FROM role_permission_grants
    JOIN permissions ON role_permission_grants.permission_id = permissions.id
    WHERE role_permission_grants.role_id = $1
"#;

pub async fn permissions_by_role_id(
    db: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<RolePermission>, sqlx::Error> {
    let rows = sqlx::query_as::<_, RolePermissionQueryResult>(QUERY)
        .bind(role_id)
        .fetch_all(db)
        .await?;

    let permissions = rows
        .into_iter()
        .map(|row| row.into())
        .collect::<Vec<RolePermission>>();

    Ok(permissions)
}

#[derive(sqlx::FromRow)]
pub struct RolePermissionQueryResult {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub permission_created_at: DateTime<Utc>,
    pub permission_updated_at: DateTime<Utc>,
}

impl From<RolePermissionQueryResult> for RolePermission {
    fn from(from: RolePermissionQueryResult) -> Self {
        Self {
            grant: RolePermissionGrant {
                role_id: from.role_id,
                permission_id: from.permission_id,
                created_at: from.created_at,
                updated_at: from.updated_at,
            },
            permission: PermissionRow {
                id: from.permission_id,
                realm: from.realm,
                resource: from.resource,
                action: from.action,
                created_at: from.permission_created_at,
                updated_at: from.permission_updated_at,
            },
        }
    }
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct RolePermission {
    pub permission: PermissionRow,
    pub grant: RolePermissionGrant,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct RolePermissionGrant {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
