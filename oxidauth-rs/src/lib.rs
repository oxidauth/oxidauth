pub mod axum;
pub mod client;
pub mod prelude;

pub use client::Client as OxidAuthClient;
pub use client::ClientError as OxidAuthClientError;
pub use client::*;

pub use oxidauth_permission as permissions;
