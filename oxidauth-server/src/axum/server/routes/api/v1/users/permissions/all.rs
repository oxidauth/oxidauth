use axum::{extract::Path, response::IntoResponse, Extension};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::{server::routes::api::v1::permissions::all::PermissionRow, Response};

#[derive(Deserialize)]
pub struct UserPermissionsAllReq {
    pub user_id: Uuid,
}

#[derive(Serialize)]
pub struct UserPermissionsAllRes {
    pub permissions: Vec<UserPermission>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserPermissionsAllReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = user_permissions_by_user_id(&mut db, params.user_id)
        .await
        .map(|permissions| UserPermissionsAllRes { permissions })
        .map_err(|error| error.to_string());

    match result {
        Ok(payload) => Response::success(payload).json(),
        Err(errors) => Response::fail(errors).json(),
    }
}

pub const QUERY: &str = r#"
    SELECT
        user_permission_grants.user_id,
        user_permission_grants.permission_id,
        user_permission_grants.created_at,
        user_permission_grants.updated_at,
        permissions.realm,
        permissions.resource,
        permissions.action,
        permissions.created_at AS permission_created_at,
        permissions.updated_at AS permission_updated_at
    FROM user_permission_grants
    JOIN permissions ON user_permission_grants.permission_id = permissions.id
    WHERE user_permission_grants.user_id = $1
"#;

pub async fn user_permissions_by_user_id(
    db: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<UserPermission>, sqlx::Error> {
    let rows = sqlx::query_as::<_, UserPermissionQueryResult>(QUERY)
        .bind(user_id)
        .fetch_all(db)
        .await?;

    let permissions = rows
        .into_iter()
        .map(|row| row.into())
        .collect::<Vec<UserPermission>>();

    Ok(permissions)
}

#[derive(sqlx::FromRow)]
pub struct UserPermissionQueryResult {
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

impl From<UserPermissionQueryResult> for UserPermission {
    fn from(from: UserPermissionQueryResult) -> Self {
        Self {
            grant: UserPermissionGrant {
                user_id: from.user_id,
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
pub struct UserPermission {
    pub permission: PermissionRow,
    pub grant: UserPermissionGrant,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPermissionGrant {
    pub user_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
