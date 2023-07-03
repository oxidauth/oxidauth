use axum::{extract::Json, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::axum::Response;

use super::all::RoleRow;

#[derive(Deserialize)]
pub struct RoleCreateReq {
    role: RoleCreateRow,
}

#[derive(Serialize)]
pub struct RoleCreateRes {
    role: RoleRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Json(req): Json<RoleCreateReq>,
) -> impl IntoResponse {
    let role = req.role;

    let result = sqlx::query_as::<_, RoleRow>(QUERY)
        .bind(role.name)
        .fetch_one(&db)
        .await;

    match result {
        Ok(role) => Response::success(RoleCreateRes { role }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    INSERT INTO roles (name)
    VALUES ($1)
    RETURNING *
"#;

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct RoleCreateRow {
    pub name: String,
}
