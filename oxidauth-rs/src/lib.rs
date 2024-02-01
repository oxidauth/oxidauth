pub mod axum;
pub mod client;
pub mod prelude;

pub use client::Client as OxidAuthClient;
pub use client::*;

/* pub use oxidauth_http::middleware; */
pub use oxidauth_permission as permissions;
