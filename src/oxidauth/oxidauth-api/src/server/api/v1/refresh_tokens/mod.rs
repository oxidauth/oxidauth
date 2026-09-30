pub mod exchange;

use axum::{Router, routing::post};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new().route("/", post(exchange::handle))
}
