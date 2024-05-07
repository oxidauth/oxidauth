pub mod generate;
pub mod validate;

use axum::{routing::post, Router};

use crate::provider::Provider;

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
