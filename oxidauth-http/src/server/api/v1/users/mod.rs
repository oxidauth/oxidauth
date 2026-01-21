pub mod authorities;
pub mod permissions;
pub mod roles;

pub mod create_user;
pub mod delete_user_by_id;
pub mod find_user_by_id;
pub mod find_user_by_username;
pub mod find_users_by_ids;
pub mod list_all_users;
pub mod update_user;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::provider::Provider;

pub const PERMISSION: &str = "oxidauth:users:manage";

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            get(list_all_users::handle),
        )
        .route(
            "/by_ids",
            post(find_users_by_ids::handle),
        )
        .route("/", post(create_user::handle))
        .route(
            "/{user_id}",
            get(find_user_by_id::handle),
        )
        .route(
            "/{user_id}",
            put(update_user::handle),
        )
        .route(
            "/{user_id}",
            delete(delete_user_by_id::handle),
        )
        .route(
            "/by_username/{username}",
            get(find_user_by_username::handle),
        )
        .nest(
            "/{user_id}/authorities",
            authorities::router(),
        )
        .nest(
            "/{user_id}/permissions",
            permissions::router(),
        )
        .nest(
            "/{user_id}/roles",
            roles::router(),
        )
}
