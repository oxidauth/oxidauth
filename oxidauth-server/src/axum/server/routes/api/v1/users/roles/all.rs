use axum::{extract::Path, response::IntoResponse, Extension};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::{server::routes::api::v1::roles::all::RoleRow, Response};

#[derive(Deserialize)]
pub struct UserRolesAllReq {
    pub user_id: Uuid,
}

#[derive(Serialize)]
pub struct UserRolesAllRes {
    pub roles: Vec<UserRole>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserRolesAllReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let result = user_roles_by_user_id(&mut db, params.user_id)
        .await
        .map(|roles| UserRolesAllRes { roles })
        .map_err(|error| error.to_string());

    match result {
        Ok(payload) => Response::success(payload).json(),
        Err(errors) => Response::fail(errors).json(),
    }
}

pub const QUERY: &str = r#"
    SELECT
        user_role_grants.user_id,
        user_role_grants.role_id,
        user_role_grants.created_at,
        user_role_grants.updated_at,
        roles.name,
        roles.created_at AS role_created_at,
        roles.updated_at AS role_updated_at
    FROM user_role_grants
    JOIN roles ON user_role_grants.role_id = roles.id
    WHERE user_role_grants.user_id = $1
"#;

pub async fn user_roles_by_user_id(
    db: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<UserRole>, sqlx::Error> {
    let result = sqlx::query_as::<_, UserRoleQueryResult>(QUERY)
        .bind(user_id)
        .fetch_all(db)
        .await?;

    let result = result
        .into_iter()
        .map(|row| row.into())
        .collect::<Vec<UserRole>>();

    Ok(result)
}

#[derive(sqlx::FromRow)]
pub struct UserRoleQueryResult {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub role_created_at: DateTime<Utc>,
    pub role_updated_at: DateTime<Utc>,
}

impl From<UserRoleQueryResult> for UserRole {
    fn from(from: UserRoleQueryResult) -> Self {
        Self {
            grant: UserRoleGrant {
                user_id: from.user_id,
                role_id: from.role_id,
                created_at: from.created_at,
                updated_at: from.updated_at,
            },
            role: RoleRow {
                id: from.role_id,
                name: from.name,
                created_at: from.role_created_at,
                updated_at: from.role_updated_at,
            },
        }
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserRole {
    pub role: RoleRow,
    pub grant: UserRoleGrant,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserRoleGrant {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
