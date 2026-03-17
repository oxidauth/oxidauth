use async_trait::async_trait;
use oxidauth_kernel::error::BoxedError;

use super::*;

#[async_trait]
pub trait AuthenticateTrait {
    async fn authenticate(&self) -> Result<bool, BoxedError>;
}

#[async_trait]
impl AuthenticateTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn authenticate(&self) -> Result<bool, BoxedError> {
        self.auth()
            .await
            .map_err(|err| err.into())
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl AuthenticateTrait for ClientMock {
    async fn authenticate(&self) -> Result<bool, BoxedError> {
        let Some(func) = self.authenticate_fn.clone() else {
            panic!("authenticate not defined for mock client");
        };

        return func();
    }
}
