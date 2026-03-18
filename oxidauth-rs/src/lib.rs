pub mod axum;
pub mod client;
pub mod prelude;

#[cfg(feature = "mock")]
pub use client::mock::ClientMock as OxidAuthClientMock;
pub use client::{
    Client as OxidAuthClient,
    ClientError as OxidAuthClientError,
    ClientTrait as OxidAuthClientTrait,
    *,
};
pub use oxidauth_kernel::{
    JsonValue,
    auth::authenticate::{
        WebhookReq,
        WebhookRes,
    },
};
pub use oxidauth_permission as permissions;
