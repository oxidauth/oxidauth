pub mod create_user_permission;
pub mod delete_user_permission;
pub mod list_user_permissions_by_user_id;

use axum::{
    Router,
    routing::{delete, get, post},
};

pub use super::PERMISSION;
use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/", get(list_user_permissions_by_user_id::handle))
        .route("/{permission}", post(create_user_permission::handle))
        .route("/{permission}", delete(delete_user_permission::handle))
}
