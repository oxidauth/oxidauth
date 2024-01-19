pub mod create_authority;
pub mod find_authority_by_id;

use axum::{routing::{delete, get, post}, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", post(create_authority::handle))
        .route("/:authority_id", get(find_authority_by_id::handle))
}

