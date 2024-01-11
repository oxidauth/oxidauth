pub mod create_user;
pub mod find_user_by_id;
pub mod find_user_by_username;
pub mod list_all_users;
pub mod update_user;

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            get(list_all_users::handle),
        )
        .route("/", post(create_user::handle))
        .route(
            "/:user_id",
            get(find_user_by_id::handle),
        )
        .route(
            "/:user_id",
            put(update_user::handle),
        )
        .route(
            "/by_username/:username",
            get(find_user_by_username::handle),
        )
}
