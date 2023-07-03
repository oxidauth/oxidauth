use axum::{extract::Path, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    authorities::{authority_factory, AuthorityStrategy},
    axum::{
        server::routes::api::v1::authorities::by_authority_strategy::authority_by_strategy,
        Response,
    },
};

use super::all::UserAuthority;

#[derive(Deserialize)]
pub struct UserAuthorityCreatePathReq {
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct UserAuthorityCreateBodyReq {
    pub authority_strategy: AuthorityStrategy,
    pub user_authority: UserAuthorityParams,
}

#[derive(Deserialize)]
pub struct UserAuthorityParams {
    pub params: Value,
}

#[derive(Serialize)]
pub struct UserAuthorityCreateRes {
    pub user_authority: UserAuthority,
}

pub async fn handler(
    Extension(db): Extension<PgPool>,
    Path(params): Path<UserAuthorityCreatePathReq>,
    Json(body): Json<UserAuthorityCreateBodyReq>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let payload = match create(&mut db, params, body).await {
        Ok(payload) => payload,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    Response::success(payload).json()
}

async fn create(
    db: &mut PgConnection,
    params: UserAuthorityCreatePathReq,
    body: UserAuthorityCreateBodyReq,
) -> Result<UserAuthorityCreateRes, String> {
    let authority = authority_by_strategy(db, &body.authority_strategy)
        .await
        .map_err(|err| err.to_string())?;

    let registrar = authority_factory(&authority, &body.authority_strategy)
        .await
        .map_err(|err| err.to_string())?;

    let user_authority = registrar
        .user_authority_from_request(body.user_authority.params)
        .await
        .map_err(|err| err.to_string())?;

    let user_authority = user_authority_create(db, params.user_id, user_authority)
        .await
        .map_err(|err| err.to_string())?;

    Ok(UserAuthorityCreateRes { user_authority })
}

const QUERY: &str = r#"
    INSERT INTO user_authorities
    (user_id, authority_id, user_identifier, params)
    VALUES($1, $2, $3, $4)
    RETURNING *
"#;

// pub async fn do_thing(pool: &PgPool, user_id: Uuid, params: UserAuthorityCreateBodyReq)  -> Result<>

pub async fn user_authority_create(
    db: &mut PgConnection,
    user_id: Uuid,
    user_authority: UserAuthorityCreate,
) -> Result<UserAuthority, sqlx::Error> {
    let user_authority = sqlx::query_as::<_, UserAuthority>(QUERY)
        .bind(user_id)
        .bind(user_authority.authority_id)
        .bind(user_authority.user_identifier)
        .bind(user_authority.params)
        .fetch_one(db)
        .await?;

    Ok(user_authority)
}

#[derive(Debug, Deserialize)]
pub struct UserAuthorityCreate {
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
}
