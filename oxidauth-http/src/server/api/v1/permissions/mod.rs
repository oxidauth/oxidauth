pub mod create_permission;
pub mod delete_permission;
pub mod find_permission_by_parts;
pub mod list_all_permissions;

use axum::{routing::{delete, get, post}, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", get(list_all_permissions::handle))
        .route("/:permission", get(find_permission_by_parts::handle))
        .route("/:permission", post(create_permission::handle))
        .route("/:permission", delete(delete_permission::handle))
}
