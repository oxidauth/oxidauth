use axum::{
    routing::{delete, get, post, put},
    Extension, Router,
};
use sqlx::PgPool;

pub mod all;
pub mod by_id;
pub mod create;
pub mod delete;
pub mod permissions;
pub mod roles;
pub mod update;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", get(all::handler))
        .route("/:role_id", get(by_id::handler))
        .route("/", post(create::handler))
        .route("/:role_id", put(update::handler))
        .route("/:role_id", delete(delete::handler))
        .layer(Extension(database.clone()))
        .nest("/:role_id/permissions", permissions::router(database))
        .nest("/:role_id/roles", roles::router(database))
}
