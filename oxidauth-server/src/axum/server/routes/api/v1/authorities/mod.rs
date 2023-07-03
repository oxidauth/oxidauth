use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};
use sqlx::PgPool;

pub mod all;
pub mod by_authority_strategy;
pub mod by_id;
pub mod create;
pub mod delete;
pub mod update;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", get(all::handler))
        .route("/", post(create::handler))
        .route("/:authority_id", get(by_id::handler))
        .route(
            "/by_strategy/:authority_strategy",
            get(by_authority_strategy::handler),
        )
        .route("/:authority_id", put(update::handler))
        .route("/:authority_id", delete(delete::handler))
        .layer(Extension(database.clone()))
}
