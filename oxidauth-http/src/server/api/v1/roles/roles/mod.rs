use axum::{routing::{delete, get, post}, Router};

use crate::provider::Provider;

pub mod create_role_role_grant;
pub mod delete_role_role_grant;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/:child_id", post(create_role_role_grant::handle))
        .route("/:child_id", delete(delete_role_role_grant::handle))
}

