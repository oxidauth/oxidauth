use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::join;
use uuid::Uuid;

use crate::axum::{
    server::routes::api::v1::{
        permissions::{
            all::PermissionRow, by_permission::QUERY as PERMISSION_BY_PERMISSION_QUERY, Permission,
        },
        roles::{all::RoleRow, by_id::QUERY as role_BY_ID_QUERY},
    },
    Response,
};

#[derive(Deserialize)]
pub struct RolePermissionDeleteReq {
    pub role_id: Uuid,
    pub permission: String,
}

#[derive(Serialize)]
pub struct RolePermissionDeleteRes {}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<RolePermissionDeleteReq>,
) -> impl IntoResponse {
    let permission = params.permission.try_into();

    let permission: Permission = match permission {
        Ok(permission) => permission,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let role_fut = sqlx::query_as::<_, RoleRow>(role_BY_ID_QUERY)
        .bind(params.role_id)
        .fetch_one(&db);

    let permission_fut = sqlx::query_as::<_, PermissionRow>(PERMISSION_BY_PERMISSION_QUERY)
        .bind(permission.realm)
        .bind(permission.resource)
        .bind(permission.action)
        .fetch_one(&db);

    let (role, permission) = join!(role_fut, permission_fut);

    let role = role.map_err(|error| format!("role not found: {}", error));

    let role = match role {
        Ok(role) => role,
        Err(errors) => return Response::fail(errors).json(),
    };

    let permission = permission.map_err(|error| format!("permission not found: {}", error));

    let permission = match permission {
        Ok(permission) => permission,
        Err(errors) => return Response::fail(errors).json(),
    };

    let grant = sqlx::query(QUERY)
        .bind(role.id)
        .bind(permission.id)
        .execute(&db)
        .await;

    let grant = grant.map_err(|error| format!("permission_grant not found: {}", error));

    match grant {
        Ok(grant) => grant,
        Err(errors) => return Response::fail(errors).json(),
    };

    Response::success(RolePermissionDeleteRes {}).json()
}

pub const QUERY: &str = r#"
    DELETE FROM role_permission_grants
    WHERE role_id = $1
    AND permission_id = $2
"#;
