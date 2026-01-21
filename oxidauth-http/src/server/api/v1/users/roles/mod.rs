pub mod create_user_role;
pub mod delete_user_role;
pub mod list_user_roles_by_user_id;

use crate::provider::Provider;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub use super::PERMISSION;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            get(list_user_roles_by_user_id::handle),
        )
        .route(
            "/{role_id}",
            post(create_user_role::handle),
        )
        .route(
            "/{role_id}",
            delete(delete_user_role::handle),
        )
}
