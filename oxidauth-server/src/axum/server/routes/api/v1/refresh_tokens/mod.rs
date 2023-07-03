use axum::{routing::post, Extension, Router};
use sqlx::PgPool;

pub mod exchange;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .route("/", post(exchange::handler))
        .layer(Extension(database.clone()))
}
