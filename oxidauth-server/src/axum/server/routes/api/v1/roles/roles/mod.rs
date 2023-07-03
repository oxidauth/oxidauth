use axum::{
    routing::{delete, post},
    Extension, Router,
};
use sqlx::PgPool;

pub mod all;
pub mod create;
pub mod delete;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/:child_id", post(create::handler))
        .route("/:child_id", delete(delete::handler))
        .layer(Extension(database.clone()))
}
