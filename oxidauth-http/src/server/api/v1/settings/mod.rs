use axum::{
    routing::{get, post, put},
    Router,
};

use crate::provider::Provider;

pub mod fetch_setting;
pub mod save_setting;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            post(save_setting::handle),
        )
        .route("/", put(save_setting::handle))
        .route(
            "/:key",
            get(fetch_setting::handle),
        )
}
