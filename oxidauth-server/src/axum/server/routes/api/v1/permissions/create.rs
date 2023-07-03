use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::axum::Response;

use super::{all::PermissionRow, Permission};

#[derive(Deserialize)]
pub struct PermissionCreateReq {
    permission: String,
}

#[derive(Serialize)]
pub struct PermissionCreateRes {
    permission: PermissionRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<PermissionCreateReq>,
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
        Ok(permission) => Response::success(PermissionCreateRes { permission }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    INSERT INTO permissions
    (realm, resource, action)
    VALUES ($1, $2, $3)
    RETURNING *
"#;

// #[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
// pub struct PermissionCreate {
//     pub realm: String,
//     pub resource: Strig,
//
//     pub action: String,
// }
