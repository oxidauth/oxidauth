use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::permissions::find_permission_by_parts::{
    FindPermissionByPartsReq,
    FindPermissionByPartsRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "find_permission_by_parts";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait FindPermissionByPartsTrait {
    async fn find_permission_by_parts<T>(
        &self,
        permission: T,
    ) -> Result<FindPermissionByPartsRes, BoxedError>
    where
        T: Into<FindPermissionByPartsReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl FindPermissionByPartsTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_permission_by_parts<T>(
        &self,
        permission: T,
    ) -> Result<FindPermissionByPartsRes, BoxedError>
    where
        T: Into<FindPermissionByPartsReq> + fmt::Debug + Send,
    {
        let permission = permission.into();

        let resp: Response<FindPermissionByPartsRes> = self
            .get(
                &format!("/permissions/{}", permission.permission),
                None::<FindPermissionByPartsReq>,
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
impl FindPermissionByPartsTrait for ClientMock {
    async fn find_permission_by_parts<T>(
        &self,
        permission: T,
    ) -> Result<FindPermissionByPartsRes, BoxedError>
    where
        T: Into<FindPermissionByPartsReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_permission_by_parts_fn
            .clone()
        else {
            panic!("find_permission_by_parts not defined for mock client");
        };

        return func(permission.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, permission};

    #[tokio::test]
    async fn find_permission_by_parts_route_contract() {
        contract(
            "GET",
            "/api/v1/permissions/oxidauth:users:read",
            ("permission", "find_permission_by_parts"),
            json!({ "permission": permission() }),
            |client| {
                async move {
                    client
                        .find_permission_by_parts(FindPermissionByPartsReq {
                            permission: "oxidauth:users:read".to_string(),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
