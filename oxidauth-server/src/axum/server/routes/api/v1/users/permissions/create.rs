use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::join;
use uuid::Uuid;

use super::all::UserPermissionGrant;
use crate::axum::{
    server::routes::api::v1::{
        permissions::{
            all::PermissionRow, by_permission::QUERY as PERMISSION_BY_PERMISSION_QUERY, Permission,
        },
        users::{all::UserRow, by_id::QUERY as USER_BY_ID_QUERY},
    },
    Response,
};

#[derive(Deserialize)]
pub struct UserPermissionCreateReq {
    pub user_id: Uuid,
    pub permission: String,
}

#[derive(Serialize)]
pub struct UserPermissionCreateRes {
    permission: PermissionRow,
    grant: UserPermissionGrant,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserPermissionCreateReq>,
) -> impl IntoResponse {
    let permission = params.permission.try_into();

    let permission: Permission = match permission {
        Ok(permission) => permission,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let user_fut = sqlx::query_as::<_, UserRow>(USER_BY_ID_QUERY)
        .bind(params.user_id)
        .fetch_one(&db);

    let permission_fut = sqlx::query_as::<_, PermissionRow>(PERMISSION_BY_PERMISSION_QUERY)
        .bind(permission.realm)
        .bind(permission.resource)
        .bind(permission.action)
        .fetch_one(&db);

    let (user, permission) = join!(user_fut, permission_fut);

    let user = user.map_err(|error| format!("user not found: {}", error));

    let user = match user {
        Ok(user) => user,
        Err(errors) => return Response::fail(errors).json(),
    };

    let permission = permission.map_err(|error| format!("permission not found: {}", error));

    let permission = match permission {
        Ok(permission) => permission,
        Err(errors) => return Response::fail(errors).json(),
    };

    let grant = sqlx::query_as::<_, UserPermissionGrant>(QUERY)
        .bind(user.id)
        .bind(permission.id)
        .fetch_one(&db)
        .await;

    let grant = grant.map_err(|error| format!("permission_grant not found: {}", error));

    let grant = match grant {
        Ok(grant) => grant,
        Err(errors) => return Response::fail(errors).json(),
    };

    Response::success(UserPermissionCreateRes { permission, grant }).json()
}

pub const QUERY: &str = r#"
    INSERT INTO user_permission_grants
    (user_id, permission_id)
    VALUES ($1, $2)
    RETURNING *
"#;
