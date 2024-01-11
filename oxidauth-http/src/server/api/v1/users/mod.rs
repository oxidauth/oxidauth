pub mod create_user;
pub mod find_user_by_id;
pub mod find_user_by_username;
pub mod list_all_users;

use axum::{
    routing::{get, post},
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
            "/by_username/:username",
            get(find_user_by_username::handle),
        )
}
