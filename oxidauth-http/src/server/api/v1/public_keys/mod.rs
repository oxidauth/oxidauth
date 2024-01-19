pub mod find_public_key_by_id;

use axum::{routing::get, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/:public_key_id",
        get(find_public_key_by_id::handle),
    )
}
