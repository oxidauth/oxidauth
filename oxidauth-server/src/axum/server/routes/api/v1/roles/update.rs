use axum::{
    extract::{Json, Path},
    response::IntoResponse,
    Extension,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

use super::{all::RoleRow, by_id};

#[derive(Deserialize)]
pub struct RoleUpdatePathReq {
    role_id: Uuid,
}

#[derive(Deserialize)]
pub struct RoleUpdateBodyReq {
    role: RoleUpdateRow,
}

#[derive(Serialize)]
pub struct RoleUpdateRes {
    role: RoleRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<RoleUpdatePathReq>,
    Json(request): Json<RoleUpdateBodyReq>,
) -> impl IntoResponse {
    let mut updates = request.role;

    updates.id = Some(params.role_id);

    let current = sqlx::query_as::<_, RoleRow>(by_id::QUERY)
        .bind(params.role_id)
        .fetch_one(&db)
        .await;

    let current = match current {
        Ok(role) => role,
        Err(_error) => return Response::fail(String::from("role not found")).json(),
    };

    if updates.name.is_none() {
        updates.name = Some(current.name);
    }

    let result = sqlx::query_as::<_, RoleRow>(QUERY)
        .bind(updates.id)
        .bind(updates.name)
        .fetch_one(&db)
        .await;

    match result {
        Ok(role) => Response::success(RoleUpdateRes { role }).json(),
        Err(error) => Response::fail(format!("error updating role: {}", error)).json(),
    }
}

const QUERY: &str = r#"
    UPDATE roles
    SET
        name = $2
    WHERE id = $1
    RETURNING *;
"#;

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct RoleUpdateRow {
    pub id: Option<Uuid>,
    pub name: Option<String>,
}
