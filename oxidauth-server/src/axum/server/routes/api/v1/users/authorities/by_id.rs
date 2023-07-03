use axum::{extract::Path, response::IntoResponse, Extension};
use serde::Deserialize;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::axum::Response;

use super::all::{UserAuthority, UserAuthorityQueryResult, UserAuthorityWithAuthority};

#[derive(Deserialize)]
pub struct UserAuthorityByIDReq {
    pub user_id: Uuid,
    pub authority_id: Uuid,
}

pub type UserAuthorityByIDRes = UserAuthorityWithAuthority;

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserAuthorityByIDReq>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, UserAuthorityQueryResult>(QUERY)
        .bind(params.user_id)
        .bind(params.authority_id)
        .fetch_one(&db)
        .await;

    let result = result
        .map(|row| {
            let user_authority: UserAuthorityByIDRes = row.into();

            user_authority
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
    AND user_authorities.authority_id = $2
"#;

pub async fn user_authority_by_user_id_and_authority_id(
    db: &mut PgConnection,
    user_id: Uuid,
    authority_id: Uuid,
) -> Result<UserAuthority, sqlx::Error> {
    let result = sqlx::query_as::<_, UserAuthorityQueryResult>(QUERY)
        .bind(user_id)
        .bind(authority_id)
        .fetch_one(db)
        .await
        .map(|row| {
            let user_authority: UserAuthorityByIDRes = row.into();

            user_authority
        })?;

    Ok(result.user_authority)
}
