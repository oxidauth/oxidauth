use axum::{
    routing::{delete, get, post},
    Extension, Router,
};
use sqlx::PgPool;

pub mod all;
pub mod create;
pub mod delete;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", get(all::handler))
        .route("/:role_id", post(create::handler))
        .route("/:role_id", delete(delete::handler))
        .route_layer(Extension(database.clone()))
}
