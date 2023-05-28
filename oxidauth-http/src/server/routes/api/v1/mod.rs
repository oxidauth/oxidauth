use axum::Router;

pub mod authenticate;
pub mod authorities;

pub fn router() -> Router {
    Router::new()
        .nest("/authenticate", authenticate::router())
        .nest("/authorities", authorities::router())
}
