pub mod auth;
pub mod permissions;
pub mod users;
pub mod roles;

use axum::Router;

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/permissions", permissions::router())
        .nest("/roles", roles::router())
}
