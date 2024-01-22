pub mod authenticate;

use axum::{routing::post, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/",
        post(authenticate::handle),
    )
}
