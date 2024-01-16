pub mod create_user_permission;
pub mod list_user_permissions_by_user_id;

use crate::provider::Provider;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            get(list_user_permissions_by_user_id::handle),
        )
        .route(
            "/:permission",
            post(create_user_permission::handle),
        )
}
