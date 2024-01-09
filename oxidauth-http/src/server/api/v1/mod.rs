pub mod auth;
pub mod users;

use axum::Router;

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
}
