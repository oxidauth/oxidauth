pub mod create_authority;

use axum::{routing::{delete, get, post}, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", post(create_authority::handle))
}

