use axum::Router;
use sqlx::PgPool;

pub mod v1;

pub fn router(database: &PgPool) -> Router {
    Router::new().nest("/v1", v1::router(database))
}
