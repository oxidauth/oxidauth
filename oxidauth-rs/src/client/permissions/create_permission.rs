use std::error::Error;

use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::permissions::create_permission::{
    CreatePermissionReq,
    CreatePermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "create_permission";

#[async_trait]
pub trait CreatePermissionTrait {
    async fn create_permission<T>(
        &self,
        permission: T,
    ) -> Result<CreatePermissionRes, BoxedError>
    where
        T: Into<CreatePermissionReq> + fmt::Debug + Send;
}

#[async_trait]
impl CreatePermissionTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_permission<T>(
        &self,
        permission: T,
    ) -> Result<CreatePermissionRes, BoxedError>
    where
        T: Into<CreatePermissionReq> + fmt::Debug + Send,
    {
        let permission = permission.into();

        let resp: Response<CreatePermissionRes> = self
            .post(
                &format!(
                    "/permissions/{}",
                    permission.permission
                ),
                None::<CreatePermissionReq>,
            )
            .await?;

        let permission_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(permission_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl CreatePermissionTrait for ClientMock {
    async fn create_permission<T>(
        &self,
        permission: T,
    ) -> Result<CreatePermissionRes, BoxedError>
    where
        T: Into<CreatePermissionReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_permission_fn
            .clone()
        else {
            panic!("create_permission not defined for mock client");
        };

        return func(permission.into());
    }
}

#[derive(Debug)]
pub struct CreatePermissionError {
    pub reason: String,
}

impl fmt::Display for CreatePermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to create permission: {}",
            self.reason
        )
    }
}

impl Error for CreatePermissionError {
}
