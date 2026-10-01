#[cfg(feature = "server")]
pub mod axum;
pub mod client;
pub mod prelude;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "mock")]
pub use client::mock::ClientMock as OxidAuthClientMock;
pub use client::*;
// The `client::*` glob above is not a server convenience: `client`'s own
// submodules import siblings through the crate root (`crate::auth::AuthTrait`,
// `crate::Client`, …), so gating it made `--no-default-features` (the wasm
// build) fail to resolve. Only the `OxidAuth*` aliases are the server-facing
// API surface.
#[cfg(feature = "server")]
pub use client::{
    Client as OxidAuthClient,
    ClientError as OxidAuthClientError,
    ClientTrait as OxidAuthClientTrait,
};
pub use oxidauth_kernel::{
    JsonValue,
    auth::authenticate::{WebhookReq, WebhookRes},
};
pub use oxidauth_permission as permissions;
