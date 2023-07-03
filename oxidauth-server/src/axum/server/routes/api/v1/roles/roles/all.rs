use axum::{extract::Path, response::IntoResponse, Extension};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::{server::routes::api::v1::roles::all::RoleRow, Response};

#[derive(Deserialize)]
pub struct RoleRolesAllReq {
    pub role_id: Uuid,
}

#[derive(Serialize)]
pub struct RoleRolesAllRes {
    pub roles: Vec<RoleRole>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<RoleRolesAllReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = roles_by_role_id(&mut db, params.role_id)
        .await
        .map(|roles| RoleRolesAllRes { roles })
        .map_err(|error| error.to_string());

    match result {
        Ok(payload) => Response::success(payload).json(),
        Err(errors) => Response::fail(errors).json(),
    }
}

pub const QUERY: &str = r#"
    SELECT
        role_role_grants.parent_id,
        role_role_grants.child_id,
        role_role_grants.created_at,
        role_role_grants.updated_at,
        roles.name,
        roles.created_at AS role_created_at,
        roles.updated_at AS role_updated_at
    FROM role_role_grants
    JOIN roles ON role_role_grants.role_id = roles.id
    WHERE role_role_grants.role_id = $1
"#;

pub async fn roles_by_role_id(
    db: &mut PgConnection,
    role_id: Uuid,
) -> Result<Vec<RoleRole>, sqlx::Error> {
    let rows = sqlx::query_as::<_, RoleRoleQueryResult>(QUERY)
        .bind(role_id)
        .fetch_all(db)
        .await?;

    let roles = rows
        .into_iter()
        .map(|row| row.into())
        .collect::<Vec<RoleRole>>();

    Ok(roles)
}

#[derive(sqlx::FromRow)]
pub struct RoleRoleQueryResult {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: String,
    pub name: String,
    pub role_created_at: DateTime<Utc>,
    pub role_updated_at: DateTime<Utc>,
}

impl From<RoleRoleQueryResult> for RoleRole {
    fn from(from: RoleRoleQueryResult) -> Self {
        Self {
            grant: RoleRoleGrant {
                parent_id: from.parent_id,
                child_id: from.child_id,
                created_at: from.created_at,
                updated_at: from.updated_at,
            },
            role: RoleRow {
                id: from.child_id,
                name: from.name,
                created_at: from.role_created_at,
                updated_at: from.role_updated_at,
            },
        }
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct RoleRole {
    pub role: RoleRow,
    pub grant: RoleRoleGrant,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct RoleRoleGrant {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
