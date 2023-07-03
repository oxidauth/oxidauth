use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::join;
use uuid::Uuid;

use super::all::UserRoleGrant;
use crate::axum::{
    server::routes::api::v1::{
        roles::{all::RoleRow, by_id::QUERY as ROLE_BY_ID_QUERY},
        users::{all::UserRow, by_id::QUERY as USER_BY_ID_QUERY},
    },
    Response,
};

#[derive(Deserialize)]
pub struct UserRoleCreateReq {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Serialize)]
pub struct UserRoleCreateRes {
    role: RoleRow,
    grant: UserRoleGrant,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserRoleCreateReq>,
) -> impl IntoResponse {
    let user_fut = sqlx::query_as::<_, UserRow>(USER_BY_ID_QUERY)
        .bind(params.user_id)
        .fetch_one(&db);

    let role_fut = sqlx::query_as::<_, RoleRow>(ROLE_BY_ID_QUERY)
        .bind(params.role_id)
        .fetch_one(&db);

    let (user, role) = join!(user_fut, role_fut);

    let user = user.map_err(|error| format!("user not found: {}", error));

    let user = match user {
        Ok(user) => user,
        Err(errors) => return Response::fail(errors).json(),
    };

    let role = role.map_err(|error| format!("role not found: {}", error));

    let role = match role {
        Ok(role) => role,
        Err(errors) => return Response::fail(errors).json(),
    };

    let grant = sqlx::query_as::<_, UserRoleGrant>(QUERY)
        .bind(user.id)
        .bind(role.id)
        .fetch_one(&db)
        .await;

    let grant = grant.map_err(|error| format!("role_grant not found: {}", error));

    let grant = match grant {
        Ok(grant) => grant,
        Err(errors) => return Response::fail(errors).json(),
    };

    Response::success(UserRoleCreateRes { role, grant }).json()
}

pub const QUERY: &str = r#"
    INSERT INTO user_role_grants
    (user_id, role_id)
    VALUES ($1, $2)
    RETURNING *
"#;
