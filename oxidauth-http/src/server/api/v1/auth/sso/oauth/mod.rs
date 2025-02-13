pub mod get_redirect_url_by_authority_client_key;

use axum::{Router, routing::post};

use crate::provider::Provider;

pub const PERMISSION: &str = "oxidauth:authorities:manage";

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/oauth",
        post(get_redirect_url_by_authority_client_key::handle),
    )
}
