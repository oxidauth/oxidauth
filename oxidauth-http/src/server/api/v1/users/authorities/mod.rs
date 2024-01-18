pub mod delete_user_authority;
pub mod update_user_authority;

use crate::provider::Provider;
use axum::{
    routing::{delete, put},
    Router,
};

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/:authority_id",
            put(update_user_authority::handle),
        )
        .route(
            "/:authority_id",
            delete(delete_user_authority::handle),
        )
}
