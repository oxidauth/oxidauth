use axum::{routing::post, Router};

use crate::provider::Provider;

pub mod validate;

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/validate",
        post(validate::handle),
    )
}
