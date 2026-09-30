//! Wire DTOs for oxidauth: request/response types shared by the
//! `oxidauth-api` server and the `oxidauth` client.
//!
//! Breaking change in 0.9.0: DTO paths dropped the `server::api::v1::`
//! prefix (they lived in the server crate before).
//! `oxidauth_http::server::api::v1::users::create_user::CreateUserReq`
//! is now `oxidauth_http::users::create_user::CreateUserReq`.

pub mod __meta;
pub mod auth;
pub mod authorities;
pub mod can;
pub mod invitations;
pub mod permissions;
pub mod public_keys;
pub mod refresh_tokens;
pub mod roles;
pub mod settings;
pub mod totp;
pub mod users;

pub use http::Response;

#[deprecated(since = "0.9.0", note = "use `oxidauth_http::Response` instead")]
pub mod response {
    pub use http::Response;
}
