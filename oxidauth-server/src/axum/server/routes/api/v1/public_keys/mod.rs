use axum::{
    routing::{delete, get, post},
    Extension, Router,
};
use sqlx::PgPool;

pub mod all;
pub mod by_id;
pub mod create;
pub mod delete;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", get(all::handler))
        .route("/", post(create::handler))
        .route("/:public_key_id", get(by_id::handler))
        .route("/:public_key_id", delete(delete::handler))
        .layer(Extension(database.clone()))
}
