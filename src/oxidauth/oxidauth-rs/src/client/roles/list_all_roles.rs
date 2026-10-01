use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::list_all_roles::{ListAllRolesReq, ListAllRolesRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "list_all_roles";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ListAllRolesTrait {
    async fn list_all_roles<T>(&self, params: T) -> Result<ListAllRolesRes, BoxedError>
    where
        T: Into<ListAllRolesReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListAllRolesTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_all_roles<T>(&self, params: T) -> Result<ListAllRolesRes, BoxedError>
    where
        T: Into<ListAllRolesReq> + fmt::Debug + Send,
    {
        let _params = params.into();

        let resp: Response<ListAllRolesRes> = self
            .get("/roles", None::<ListAllRolesReq>)
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListAllRolesTrait for ClientMock {
    async fn list_all_roles<T>(&self, params: T) -> Result<ListAllRolesRes, BoxedError>
    where
        T: Into<ListAllRolesReq> + fmt::Debug + Send,
    {
        let Some(func) = self.list_all_roles_fn.clone() else {
            panic!("list_all_roles not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, role};

    #[tokio::test]
    async fn list_all_roles_route_contract() {
        contract(
            "GET",
            "/api/v1/roles",
            ("role", "list_all_roles"),
            json!({ "roles": [role()] }),
            |client| {
                async move {
                    client
                        .list_all_roles(ListAllRolesReq {})
                        .await
                }
            },
        )
        .await;
    }
}
