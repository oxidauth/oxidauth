pub mod api;

use axum::Router;

pub fn router() -> Router {
    Router::new()
}
