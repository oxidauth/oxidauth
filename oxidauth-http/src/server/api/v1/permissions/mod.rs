pub mod create_permission;

use axum::{routing::post, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", post(create_permission::handle))
}
