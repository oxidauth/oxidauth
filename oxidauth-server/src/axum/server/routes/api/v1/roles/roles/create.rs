use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::join;
use uuid::Uuid;

use super::all::RoleRoleGrant;
use crate::axum::{
    server::routes::api::v1::roles::{all::RoleRow, by_id::QUERY as ROLE_BY_ID_QUERY},
    Response,
};

#[derive(Deserialize)]
pub struct RoleRoleCreateReq {
    pub role_id: Uuid,
    pub child_id: Uuid,
}

#[derive(Serialize)]
pub struct RoleRoleCreateRes {
    child: RoleRow,
    grant: RoleRoleGrant,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<RoleRoleCreateReq>,
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

    let grant = sqlx::query_as::<_, RoleRoleGrant>(QUERY)
        .bind(role.id)
        .bind(child.id)
        .fetch_one(&db)
        .await;

    let grant = grant.map_err(|error| format!("child_grant not found: {}", error));

    let grant = match grant {
        Ok(grant) => grant,
        Err(errors) => return Response::fail(errors).json(),
    };

    Response::success(RoleRoleCreateRes { child, grant }).json()
}

pub const QUERY: &str = r#"
    INSERT INTO role_role_grants
    (parent_id, child_id)
    VALUES ($1, $2)
    RETURNING *
"#;
