use axum::{
    routing::{delete, get, post},
    Extension, Router,
};
use sqlx::PgPool;

pub mod permission;
pub use permission::Permission;

pub mod all;
pub mod by_permission;
pub mod create;
pub mod delete;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", get(all::handler))
        .route("/:permission", get(by_permission::handler))
        .route("/:permission", post(create::handler))
        .route("/:permission", delete(delete::handler))
        .layer(Extension(database.clone()))
}
