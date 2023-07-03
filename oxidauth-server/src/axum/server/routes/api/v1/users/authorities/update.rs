use axum::{extract::Path, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::axum::Response;

use super::all::UserAuthority;

#[derive(Deserialize)]
pub struct UserAuthorityUpdatePathReq {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}

#[derive(Deserialize)]
pub struct UserAuthorityUpdateBodyReq {
    pub user_authority: UserAuthorityUpdate,
}

#[derive(Serialize)]
pub struct UserAuthorityUpdateRes {
    pub user_authority: UserAuthority,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserAuthorityUpdatePathReq>,
    Json(body): Json<UserAuthorityUpdateBodyReq>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, UserAuthority>(QUERY)
        .bind(params.user_id)
        .bind(params.authority_id)
        .bind(body.user_authority.params)
        .fetch_one(&db)
        .await;

    let result = result
        .map(|user_authority| UserAuthorityUpdateRes { user_authority })
        .map_err(|error| error.to_string());

    match result {
        Ok(payload) => Response::success(payload).json(),
        Err(errors) => Response::fail(errors).json(),
    }
}

pub const QUERY: &str = r#"
    UPDATE user_authorities
    SET params = $3
    WHERE user_id = $1
    AND authority_id = $2
    RETURNING *
"#;

#[derive(Deserialize)]
pub struct UserAuthorityUpdate {
    pub params: Value,
}
