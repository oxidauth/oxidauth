pub mod find_redirect_url_by_authority_client_key;

use axum::{routing::post, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/oauth",
        post(find_redirect_url_by_authority_client_key::handler()),
    )
}
