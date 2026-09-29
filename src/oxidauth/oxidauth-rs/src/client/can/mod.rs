use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::can::CanReq;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "can";

#[async_trait]
pub trait CanTrait {
    async fn can<T>(&self, params: T) -> Result<bool, BoxedError>
    where
        T: Into<CanReq> + fmt::Debug + Send;
}

#[async_trait]
impl CanTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn can<T>(&self, params: T) -> Result<bool, BoxedError>
    where
        T: Into<CanReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<bool> = self
            .get(
                &format!("/can/{}", params.permission),
                None::<CanReq>,
            )
            .await?;

        let can_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(can_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl CanTrait for ClientMock {
    async fn can<T>(&self, params: T) -> Result<bool, BoxedError>
    where
        T: Into<CanReq> + fmt::Debug + Send,
    {
        let Some(func) = self.can_fn.clone() else {
            panic!("can not defined for mock client");
        };

        return func(params.into());
    }
}
