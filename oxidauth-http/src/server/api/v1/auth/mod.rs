pub mod authenticate;
pub mod oauth2;
pub mod register;

use axum::{Router, routing::post};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .nest("/oauth2", oauth2::router())
        .route(
            "/authenticate",
            post(authenticate::handle),
        )
        .route(
            "/register",
            post(register::handle),
        )
}
