use axum::{routing::post, Router};

use crate::provider::Provider;

pub mod generate;
pub mod validate;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/generate",
            post(generate::handle),
        )
        .route(
            "/validate",
            post(validate::handle),
        )
}
