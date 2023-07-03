use axum::{extract::Path, response::IntoResponse, Extension};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

#[derive(Deserialize)]
pub struct UserAuthoritiesAllReq {
    pub user_id: Uuid,
}

#[derive(Serialize)]
pub struct UserAuthoritiesAllRes {
    pub user_authorities: Vec<UserAuthorityWithAuthority>,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserAuthoritiesAllReq>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, UserAuthorityQueryResult>(QUERY)
        .bind(params.user_id)
        .fetch_all(&db)
        .await;

    let result = result
        .map(|rows| {
            let user_authorities = rows
                .into_iter()
                .map(|row| row.into())
                .collect::<Vec<UserAuthorityWithAuthority>>();

            UserAuthoritiesAllRes { user_authorities }
        })
        .map_err(|error| error.to_string());

    match result {
        Ok(payload) => Response::success(payload).json(),
        Err(errors) => Response::fail(errors).json(),
    }
}

pub const QUERY: &str = r#"
    SELECT
        user_authorities.user_id,
        user_authorities.authority_id,
        user_authorities.user_identifier,
        user_authorities.params,
        user_authorities.created_at,
        user_authorities.updated_at,
        authorities.name AS authority_name,
        authorities.strategy AS authority_strategy,
        authorities.created_at AS authority_created_at,
        authorities.updated_at AS authority_updated_at
    FROM user_authorities
    JOIN authorities ON user_authorities.authority_id = authorities.id
    WHERE user_authorities.user_id = $1
"#;

#[derive(sqlx::FromRow)]
pub struct UserAuthorityQueryResult {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authority_name: String,
    pub authority_strategy: String,
    pub authority_created_at: DateTime<Utc>,
    pub authority_updated_at: DateTime<Utc>,
}

impl From<UserAuthorityQueryResult> for UserAuthorityWithAuthority {
    fn from(from: UserAuthorityQueryResult) -> Self {
        Self {
            user_authority: UserAuthority {
                user_id: from.user_id,
                authority_id: from.authority_id,
                user_identifier: from.user_identifier,
                params: from.params,
                created_at: from.created_at,
                updated_at: from.updated_at,
            },
            authority: Authority {
                id: from.authority_id,
                name: from.authority_name,
                strategy: from.authority_strategy,
                created_at: from.authority_created_at,
                updated_at: from.authority_updated_at,
            },
        }
    }
}

#[derive(Serialize)]
pub struct UserAuthorityWithAuthority {
    pub user_authority: UserAuthority,
    pub authority: Authority,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct Authority {
    pub id: Uuid,
    pub name: String,
    pub strategy: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
