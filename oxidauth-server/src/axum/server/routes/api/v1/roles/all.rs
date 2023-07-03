use axum::{extract::Query, response::IntoResponse, Extension};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct RolesAllReq {}

#[derive(Serialize)]
pub struct RolesAllRes {
    roles: Vec<RoleRow>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Query(_params): Query<RolesAllReq>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, RoleRow>(QUERY).fetch_all(&db).await;

    match result {
        Ok(roles) => Response::success(RolesAllRes { roles }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    SELECT * FROM roles
"#;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
