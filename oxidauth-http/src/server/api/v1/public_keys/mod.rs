pub mod create_public_key;
pub mod find_public_key_by_id;
pub mod list_all_public_keys;

use axum::{routing::get, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            get(list_all_public_keys::handle),
        )
        .route(
            "/:public_key_id",
            get(find_public_key_by_id::handle),
        )
}
