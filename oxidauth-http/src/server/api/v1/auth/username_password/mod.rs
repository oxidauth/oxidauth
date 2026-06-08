pub mod forgot_password;
pub mod reset_password;

use axum::{Router, routing::post};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/forgot_password", post(forgot_password::handle))
        .route("/reset_password", post(reset_password::handle))
}
