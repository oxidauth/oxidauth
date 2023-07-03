use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::axum::Response;

use super::Permission;

#[derive(Deserialize)]
pub struct PermissionDeleteReq {
    permission: String,
}

#[derive(Serialize)]
pub struct PermissionDeleteRes {}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<PermissionDeleteReq>,
) -> impl IntoResponse {
    let permission = params.permission.try_into();

    let permission: Permission = match permission {
        Ok(permission) => permission,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = sqlx::query(QUERY)
        .bind(permission.realm)
        .bind(permission.resource)
        .bind(permission.action)
        .execute(&db)
        .await;

    match result {
        Ok(_) => Response::<(), _>::default().json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    DELETE FROM permissions
    WHERE realm = $1
    AND resource = $2
    AND action = $3
"#;
