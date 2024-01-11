pub mod create_permission;
pub mod find_permission_by_parts;

use axum::{routing::{get, post}, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/:permission", get(find_permission_by_parts::handle))
        .route("/:permission", post(create_permission::handle))
}
