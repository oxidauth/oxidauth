use std::error::Error;

use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::permissions::create_permission::{CreatePermissionReq, CreatePermissionRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "create_permission";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CreatePermissionTrait {
    async fn create_permission<T>(&self, permission: T) -> Result<CreatePermissionRes, BoxedError>
    where
        T: Into<CreatePermissionReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreatePermissionTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_permission<T>(&self, permission: T) -> Result<CreatePermissionRes, BoxedError>
    where
        T: Into<CreatePermissionReq> + fmt::Debug + Send,
    {
        let permission = permission.into();

        let resp: Response<CreatePermissionRes> = self
            .post(
                &format!("/permissions/{}", permission.permission),
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
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreatePermissionTrait for ClientMock {
    async fn create_permission<T>(&self, permission: T) -> Result<CreatePermissionRes, BoxedError>
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
        write!(f, "unable to create permission: {}", self.reason)
    }
}

impl Error for CreatePermissionError {
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, permission};

    #[tokio::test]
    async fn create_permission_route_contract() {
        contract(
            "POST",
            "/api/v1/permissions/oxidauth:users:read",
            ("permission", "create_permission"),
            json!({ "permission": permission() }),
            |client| {
                async move {
                    client
                        .create_permission(CreatePermissionReq {
                            permission: "oxidauth:users:read".to_string(),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
