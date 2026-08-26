use async_trait::async_trait;

use super::*;

#[async_trait(?Send)]
#[async_trait]
pub trait AuthenticateTrait {
    async fn authenticate(
        &self,
        client_key: Uuid,
        username: &str,
        password: &str,
    ) -> Result<bool, String>;
}

#[async_trait(?Send)]
#[async_trait]
impl AuthenticateTrait for Client {
    async fn authenticate(
        &self,
        client_key: Uuid,
        username: &str,
        password: &str,
    ) -> Result<bool, String> {
        self.auth(client_key, username, password)
            .await
            .map_err(|_| "authentication failed".to_string())
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl AuthenticateTrait for ClientMock {
    async fn authenticate(&self, username: &str, password: &str) -> Result<bool, String> {
        let Some(func) = self.authenticate_fn.clone() else {
            panic!("authenticate not defined for mock client");
        };

        return func();
    }
}
