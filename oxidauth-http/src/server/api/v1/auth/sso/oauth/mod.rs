pub mod get_redirect_url_by_authority_client_key;

use axum::{Router, routing::post};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/build_redirect",
        post(get_redirect_url_by_authority_client_key::handle),
    )
}
