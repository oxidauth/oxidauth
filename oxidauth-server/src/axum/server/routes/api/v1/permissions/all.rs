use axum::{extract::Query, response::IntoResponse, Extension};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct PermissionsAllReq {}

#[derive(Serialize)]
pub struct PermissionsAllRes {
    permissions: Vec<PermissionRow>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Query(_params): Query<PermissionsAllReq>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, PermissionRow>(QUERY)
        .fetch_all(&db)
        .await;

    match result {
        Ok(permissions) => Response::success(PermissionsAllRes { permissions }).json(),
        Err(error) => Response::fail(error.to_string()).json(),
    }
}

const QUERY: &str = r#"
    SELECT * FROM permissions
"#;

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PermissionRow {
    pub id: Uuid,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ToString for PermissionRow {
    fn to_string(&self) -> String {
        format!("{}:{}:{}", self.realm, self.resource, self.action)
    }
}
