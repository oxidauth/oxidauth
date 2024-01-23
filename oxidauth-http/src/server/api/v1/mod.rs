pub mod auth;
pub mod authorities;
pub mod permissions;
pub mod public_keys;
pub mod refresh_tokens;
pub mod roles;
pub mod users;

use axum::Router;

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .nest("/auth", auth::router())
        .nest(
            "/authorities",
            authorities::router(),
        )
        .nest("/users", users::router())
        .nest(
            "/permissions",
            permissions::router(),
        )
        .nest(
            "/public_keys",
            public_keys::router(),
        )
        .nest("/refresh_tokens", refresh_tokens::router())
        .nest("/roles", roles::router())
        .nest(
            "/refresh_tokens",
            refresh_tokens::router(),
        )
}
