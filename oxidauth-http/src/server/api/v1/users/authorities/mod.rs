pub mod update_user_authority;

use crate::provider::Provider;
use axum::{routing::put, Router};

pub fn router() -> Router<Provider> {
    Router::new().route(
        "/:authority_id",
        put(update_user_authority::handle),
    )
}
