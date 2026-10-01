use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::find_role_by_name::FindRoleByNameRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "find_role_by_name";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait FindRoleByNameTrait {
    async fn find_role_by_name<T>(&self, role: T) -> Result<FindRoleByNameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl FindRoleByNameTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_role_by_name<T>(&self, role: T) -> Result<FindRoleByNameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let role = role.into();

        let resp: Response<FindRoleByNameRes> = self
            .get(&format!("/roles/by_name/{}", role), None::<()>)
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
impl FindRoleByNameTrait for ClientMock {
    async fn find_role_by_name<T>(&self, role: T) -> Result<FindRoleByNameRes, BoxedError>
    where
        T: Into<String> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_role_by_name_fn
            .clone()
        else {
            panic!("find_role_by_name not defined for mock client");
        };

        return func(role.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, role};

    #[tokio::test]
    async fn find_role_by_name_route_contract() {
        contract(
            "GET",
            "/api/v1/roles/by_name/admin",
            ("role", "find_role_by_name"),
            json!({ "role": role() }),
            |client| {
                async move {
                    client
                        .find_role_by_name("admin".to_string())
                        .await
                }
            },
        )
        .await;
    }
}
