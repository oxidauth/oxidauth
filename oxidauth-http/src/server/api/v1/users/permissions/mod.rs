pub mod create_user_permission;

use crate::provider::Provider;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/:permission",
        post(create_user_permission::handle),
    )
}
