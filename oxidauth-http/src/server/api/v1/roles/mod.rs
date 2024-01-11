pub mod create_role;
pub mod delete_role;
pub mod find_role_by_id;
pub mod list_all_roles;
pub mod update_role;

use axum::{routing::{delete, get, post}, Router};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", get(list_all_roles::handle))
        .route("/", post(create_role::handle))
        .route("/:role_id", get(find_role_by_id::handle))
        .route("/:role_id", post(update_role::handle))
        .route("/:role_id", delete(delete_role::handle))
}
