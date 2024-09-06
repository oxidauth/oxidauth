pub mod create_role;
pub mod delete_role;
pub mod find_role_by_id;
pub mod find_role_by_name;
pub mod list_all_roles;
pub mod permissions;
#[allow(clippy::module_inception)]
pub mod roles;
pub mod update_role;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::provider::Provider;

pub const PERMISSION: &str = "oxidauth:roles:manage";

pub fn router() -> Router<Provider> {
    Router::new()
        .route(
            "/",
            get(list_all_roles::handle),
        )
        .route("/", post(create_role::handle))
        .route(
            "/:role_id",
            get(find_role_by_id::handle),
        )
        .route(
            "/:role_id",
            put(update_role::handle),
        )
        .route(
            "/:role_id",
            delete(delete_role::handle),
        )
        .route(
            "/by_name/:role",
            get(find_role_by_name::handle),
        )
        .nest(
            "/:parent_id/roles",
            roles::router(),
        )
        .nest(
            "/:role_id/permissions",
            permissions::router(),
        )
}
