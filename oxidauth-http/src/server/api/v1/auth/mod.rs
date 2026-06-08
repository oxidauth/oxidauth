pub mod authenticate;
pub mod oauth2;
pub mod register;
pub mod username_password;

use axum::{Router, routing::post};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .nest("/oauth2", oauth2::router())
        .nest("/username_password", username_password::router())
        .route("/authenticate", post(authenticate::handle))
        .route("/register", post(register::handle))
}
