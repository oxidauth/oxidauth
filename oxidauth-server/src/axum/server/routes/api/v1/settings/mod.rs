use axum::Router;
use sqlx::PgPool;

pub mod fetch;
pub mod save;

pub fn router(_db: &PgPool) -> Router {
    Router::new()
}
