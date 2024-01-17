use axum::{routing::{delete, get, post}, Router};

use crate::provider::Provider;

pub mod create_role_permission_grant;

pub fn router() -> Router<Provider> {
    Router::new()
        .route("/:permission", post(create_role_permission_grant::handle))
}


