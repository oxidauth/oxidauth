use axum::{extract::Path, response::IntoResponse, Extension};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct RoleDeleteByIDReq {
    role_id: Uuid,
}

#[derive(Serialize)]
pub struct RoleDeleteByIDRes {}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<RoleDeleteByIDReq>,
) -> impl IntoResponse {
    let result = sqlx::query(QUERY).bind(params.role_id).execute(&db).await;

    match result {
        Ok(_) => Response::<(), _>::default().json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    DELETE FROM roles
    WHERE id = $1
"#;
