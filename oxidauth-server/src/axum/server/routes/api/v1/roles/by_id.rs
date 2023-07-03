use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::Response;

use super::all::RoleRow;

#[derive(Deserialize)]
pub struct RoleByIDReq {
    role_id: Uuid,
}

#[derive(Serialize)]
pub struct RoleByIDRes {
    role: RoleRow,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<RoleByIDReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = role_by_id(&mut db, params.role_id).await;

    match result {
        Ok(role) => Response::success(RoleByIDRes { role }).json(),
        Err(error) => Response::fail(format!("role not found: {}", error.to_string())).json(),
    }
}

pub const QUERY: &str = r#"
    SELECT *
    FROM roles
    WHERE id = $1
"#;

pub async fn role_by_id(db: &mut PgConnection, role_id: Uuid) -> Result<RoleRow, sqlx::Error> {
    let result = sqlx::query_as::<_, RoleRow>(QUERY)
        .bind(role_id)
        .fetch_one(db)
        .await?;

    Ok(result)
}
