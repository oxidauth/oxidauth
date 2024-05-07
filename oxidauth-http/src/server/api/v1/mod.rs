pub mod auth;
pub mod authorities;
pub mod can;
pub mod invitations;
pub mod meta;
pub mod permissions;
pub mod public_keys;
pub mod refresh_tokens;
pub mod roles;
pub mod settings;
pub mod totp;
pub mod users;

use axum::Router;

use crate::provider::Provider;

pub fn router() -> Router<Provider> {
    Router::new()
        .nest("/__meta", meta::router())
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
        .nest("/roles", roles::router())
        .nest(
            "/refresh_tokens",
            refresh_tokens::router(),
        )
        .nest("/can", can::router())
        .nest(
            "/settings",
            settings::router(),
        )
        .nest(
            "/invitations",
            invitations::router(),
        )
}
