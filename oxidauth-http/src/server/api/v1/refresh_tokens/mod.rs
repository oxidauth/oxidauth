pub mod exchange;

use axum::{routing::post, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new().route("/", post(exchange::handle))
}
