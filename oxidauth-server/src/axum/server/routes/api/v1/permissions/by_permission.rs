use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::axum::Response;

use super::{all::PermissionRow, Permission};

#[derive(Deserialize)]
pub struct PermissionByIDReq {
    permission: String,
}

#[derive(Serialize)]
pub struct PermissionByIDRes {
    permission: PermissionRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<PermissionByIDReq>,
) -> impl IntoResponse {
    let permission = params.permission.try_into();

    let permission: Permission = match permission {
        Ok(permission) => permission,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = sqlx::query_as::<_, PermissionRow>(QUERY)
        .bind(permission.realm)
        .bind(permission.resource)
        .bind(permission.action)
        .fetch_one(&db)
        .await;

    match result {
        Ok(permission) => Response::success(PermissionByIDRes { permission }).json(),
        Err(error) => {
            return Response::fail(format!("permission not found: {}", error.to_string())).json()
        }
    }
}

pub const QUERY: &str = r#"
    SELECT *
    FROM permissions
    WHERE realm = $1
    AND resource = $2
    AND action = $3
"#;
