use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::join;
use uuid::Uuid;

use crate::axum::{
    server::routes::api::v1::roles::{all::RoleRow, by_id::QUERY as ROLE_BY_ID_QUERY},
    Response,
};

#[derive(Deserialize)]
pub struct RoleRoleDeleteReq {
    pub role_id: Uuid,
    pub child_id: Uuid,
}

#[derive(Serialize)]
pub struct RoleRoleDeleteRes {}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<RoleRoleDeleteReq>,
) -> impl IntoResponse {
    let role_fut = sqlx::query_as::<_, RoleRow>(ROLE_BY_ID_QUERY)
        .bind(params.role_id)
        .fetch_one(&db);

    let child_fut = sqlx::query_as::<_, RoleRow>(ROLE_BY_ID_QUERY)
        .bind(params.child_id)
        .fetch_one(&db);

    let (role, child) = join!(role_fut, child_fut);

    let role = role.map_err(|error| format!("role not found: {}", error));

    let role = match role {
        Ok(role) => role,
        Err(errors) => return Response::fail(errors).json(),
    };

    let child = child.map_err(|error| format!("child not found: {}", error));

    let child = match child {
        Ok(child) => child,
        Err(errors) => return Response::fail(errors).json(),
    };

    let grant = sqlx::query(QUERY)
        .bind(role.id)
        .bind(child.id)
        .execute(&db)
        .await;

    let grant = grant.map_err(|error| format!("child_grant not found: {}", error));

    match grant {
        Ok(grant) => grant,
        Err(errors) => return Response::fail(errors).json(),
    };

    Response::success(RoleRoleDeleteRes {}).json()
}

pub const QUERY: &str = r#"
    DELETE FROM role_role_grants
    WHERE parent_id = $1
    AND child_id = $2
"#;
