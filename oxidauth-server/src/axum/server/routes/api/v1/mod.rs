use axum::Router;
use sqlx::PgPool;

pub mod authenticate;
pub mod authorities;
pub mod can;
pub mod invitations;
pub mod permissions;
pub mod public_keys;
pub mod refresh_tokens;
pub mod register;
pub mod roles;
pub mod settings;
pub mod users;

pub fn router(database: &PgPool) -> Router {
    Router::new()
        .nest("/authenticate", authenticate::router(database))
        .nest("/authorities", authorities::router(database))
        .nest("/can", can::router(database))
        .nest("/permissions", permissions::router(database))
        .nest("/public_keys", public_keys::router(database))
        .nest("/refresh_tokens", refresh_tokens::router(database))
        .nest("/register", register::router(database))
        .nest("/roles", roles::router(database))
        .nest("/settings", settings::router(database))
        .nest("/users", users::router(database))
        .nest("/invitations", invitations::router(database))
}
