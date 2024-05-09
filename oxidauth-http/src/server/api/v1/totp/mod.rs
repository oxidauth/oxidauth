pub mod validate;

use axum::{routing::post, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/validate",
        post(validate::handle),
    )
}
