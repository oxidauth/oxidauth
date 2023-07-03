use axum::{response::IntoResponse, routing::post, Extension, Json, Router};
use sqlx::PgPool;

use crate::{
    authorities::{authenticate, Request as AuthRequest},
    axum::Response,
};

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", post(handle_authenticate))
        .layer(Extension(database.clone()))
}

pub async fn handle_authenticate(
    Extension(db): Extension<PgPool>,
    Json(params): Json<AuthRequest>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let payload = match authenticate(&mut db, params).await {
        Ok(payload) => payload,
        Err(_error) => return Response::fail("unable to authenticate".to_string()).json(),
    };

    Response::success(payload).json()
}
