use async_trait::async_trait;
use oxidauth_kernel::error::BoxedError;

use super::*;

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait AuthenticateTrait {
    async fn authenticate(&self, username: &str, password: &str) -> Result<bool, BoxedError>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl AuthenticateTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn authenticate(&self, username: &str, password: &str) -> Result<bool, BoxedError> {
        self.auth(username, password)
            .await
            .map_err(|err| err.into())
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl AuthenticateTrait for ClientMock {
    async fn authenticate(&self, _username: &str, _password: &str) -> Result<bool, BoxedError> {
        let Some(func) = self.authenticate_fn.clone() else {
            panic!("authenticate not defined for mock client");
        };

        return func();
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use wiremock::{
        Mock,
        MockServer,
        ResponseTemplate,
        matchers::{method, path},
    };

    use super::*;
    use crate::client::{ClientError, ClientErrorKind};

    // AuthenticateTrait::authenticate has no route of its own — the POST
    // /auth/authenticate + GET /public_keys contract is pinned by the E1/E2
    // tests in `client/mod.rs`. This file's only logic is delegating to
    // `Client::auth` and re-typing ClientError into the trait's BoxedError;
    // this test pins that conversion survives the delegation.
    #[tokio::test]
    async fn authenticate_delegates_to_auth_and_retypes_client_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/public_keys"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>"))
            .mount(&server)
            .await;

        let client = Client::new(
            &url::Url::parse(&server.uri()).unwrap(),
            uuid::Uuid::new_v4(),
        )
        .unwrap();

        // BUG(pinned): the inherent `Client::authenticate` in client/mod.rs
        // shadows this trait method at any `client.authenticate(..)` call
        // site, so the trait's BoxedError variant is only reachable through
        // qualified syntax — and the two entry points return different error
        // types for the same operation.
        let err = <Client as AuthenticateTrait>::authenticate(&client, "malreynolds", "wrong")
            .await
            .expect_err("non-JSON keys response must sink auth");

        let client_err = err
            .downcast_ref::<ClientError>()
            .expect("authenticate must surface the underlying ClientError");
        assert!(matches!(
            client_err.kind,
            ClientErrorKind::Other("unable to deserialize public keys")
        ));
    }
}
