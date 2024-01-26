use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::provider::Provider;

pub mod create_role_permission_grant;
pub mod delete_role_permission_grant;
pub mod list_role_permission_grants_by_role_id;

pub use super::PERMISSION;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            get(list_role_permission_grants_by_role_id::handle),
        )
        .route(
            "/:permission",
            post(create_role_permission_grant::handle),
        )
        .route(
            "/:permission",
            delete(delete_role_permission_grant::handle),
        )
}
