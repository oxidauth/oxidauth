pub mod create_user_authority;
pub mod delete_user_authority;
pub mod find_user_authority_by_user_id_and_authority_id;
pub mod list_user_authorities_by_user_id;
pub mod update_user_authority;

use crate::provider::Provider;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            get(list_user_authorities_by_user_id::handle),
        )
        .route(
            "/",
            post(create_user_authority::handle),
        )
        .route(
            "/:authority_id",
            get(find_user_authority_by_user_id_and_authority_id::handle),
        )
        .route(
            "/:authority_id",
            put(update_user_authority::handle),
        )
        .route(
            "/:authority_id",
            delete(delete_user_authority::handle),
        )
}
