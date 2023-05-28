use axum::Router;

pub mod v1;

pub fn router() -> Router {
    Router::new().nest("/v1", v1::router())
}
