use axum::{response::IntoResponse, routing::post, Extension, Json, Router};
use sqlx::PgPool;

use crate::{
    authorities::{register, Request as AuthRequest},
    axum::Response,
};

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", post(handle_register))
        .layer(Extension(database.clone()))
}

async fn handle_register(
    Extension(db): Extension<PgPool>,
    Json(params): Json<AuthRequest>,
) -> impl IntoResponse {
    let mut db = match db.acquire().await {
        Ok(db) => db,
        Err(error) => return Response::fail(error.to_string()).json(),
    };

    let payload = match register(&mut db, params).await {
        Ok(payload) => payload,
        Err(error) => {
            return Response::fail(format!("unable to register: {}", error)).json();
        }
    };

    Response::success(payload).json()
}
