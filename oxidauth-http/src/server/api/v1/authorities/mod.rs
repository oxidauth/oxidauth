pub mod create_authority;
pub mod delete_authority;
pub mod find_authority_by_strategy;
pub mod find_authority_by_id;
pub mod list_all_authorities;
pub mod update_authority;

use axum::{routing::{delete, get, post}, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", get(list_all_authorities::handle))
        .route("/", post(create_authority::handle))
        .route("/by_strategy/:strategy", get(find_authority_by_strategy::handle))
        .route("/:authority_id", get(find_authority_by_id::handle))
        .route("/:authority_id", post(update_authority::handle))
        .route("/:authority_id", delete(delete_authority::handle))
}

