use axum::{
    routing::{post, put},
    Router,
};

use crate::provider::Provider;

pub mod save_setting;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            post(save_setting::handle),
        )
        .route("/", put(save_setting::handle))
}
