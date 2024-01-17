pub mod create_user_role;

use crate::provider::Provider;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/:role_id",
        post(create_user_role::handle),
    )
}
