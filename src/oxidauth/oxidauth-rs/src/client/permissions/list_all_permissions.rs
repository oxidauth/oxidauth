use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::permissions::list_all_permissions::{
    ListAllPermissionsReq,
    ListAllPermissionsRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "list_all_permissions";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ListAllPermissionsTrait {
    async fn list_all_permissions<T>(&self, params: T) -> Result<ListAllPermissionsRes, BoxedError>
    where
        T: Into<ListAllPermissionsReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListAllPermissionsTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_permissions<T>(&self, params: T) -> Result<ListAllPermissionsRes, BoxedError>
    where
        T: Into<ListAllPermissionsReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListAllPermissionsRes> = self
            .get("/permissions", params)
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
impl ListAllPermissionsTrait for ClientMock {
    async fn list_all_permissions<T>(&self, params: T) -> Result<ListAllPermissionsRes, BoxedError>
    where
        T: Into<ListAllPermissionsReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .list_all_permissions_fn
            .clone()
        else {
            panic!("list_all_permissions not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use oxidauth_kernel::permissions::list_all_permissions::ListAllPermissions;
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, permission};

    #[tokio::test]
    async fn list_all_permissions_route_contract() {
        contract(
            "GET",
            "/api/v1/permissions",
            ("permission", "list_all_permissions"),
            json!({ "permissions": [permission()] }),
            |client| {
                async move {
                    client
                        .list_all_permissions(ListAllPermissions)
                        .await
                }
            },
        )
        .await;
    }
}
