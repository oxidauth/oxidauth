pub mod authenticate;
pub mod register;
pub mod sso;

use axum::{routing::post, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .nest("/sso", sso::router())
        .route(
            "/authenticate",
            post(authenticate::handle),
        )
        .route(
            "/register",
            post(register::handle),
        )
}
