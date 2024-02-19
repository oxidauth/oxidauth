use std::error::Error;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::permissions::create_permission::{
    CreatePermissionReq, CreatePermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "create_permission";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn create_permission<T>(
        &self,
        permission: T,
    ) -> Result<CreatePermissionRes, BoxedError>
    where
        T: Into<CreatePermissionReq> + fmt::Debug,
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

impl Error for CreatePermissionError {}
