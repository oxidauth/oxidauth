use axum::Router;

pub mod api;

pub fn router() -> Router {
    Router::new().nest("/api", api::router())
}
