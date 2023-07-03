use axum::{extract::Query, response::IntoResponse, Extension};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct UsersAllReq {}

#[derive(Serialize)]
pub struct UsersAllRes {
    users: Vec<UserRow>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Query(_params): Query<UsersAllReq>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, UserRow>(QUERY).fetch_all(&db).await;

    match result {
        Ok(users) => Response::success(UsersAllRes { users }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    SELECT * FROM users
"#;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: String,
    pub kind: String,
    pub profile: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
