pub mod authenticate;
pub mod register;

use axum::{routing::post, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            post(authenticate::handle),
        )
        .route(
            "/register",
            post(register::handle),
        )
}
