pub mod create_authority;
pub mod delete_authority;
pub mod find_authority_by_id;
pub mod find_authority_by_strategy;
pub mod list_all_authorities;
pub mod update_authority;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::provider::Provider;

pub const PERMISSION: &str = "oxidauth:authorities:manage";

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", get(list_all_authorities::handle))
        .route("/", post(create_authority::handle))
        .route(
            "/by_strategy/:strategy",
            get(find_authority_by_strategy::handle),
        )
        .route("/:authority_id", get(find_authority_by_id::handle))
        .route("/:authority_id", put(update_authority::handle))
        .route("/:authority_id", delete(delete_authority::handle))
}
