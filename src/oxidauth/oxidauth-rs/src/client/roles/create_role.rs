use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::create_role::{CreateRoleReq, CreateRoleRes};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::roles::create_role::CreateRole;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "create_role";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CreateRoleTrait {
    async fn create_role<T>(&self, role: T) -> Result<CreateRoleRes, BoxedError>
    where
        T: Into<CreateRoleReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_role<T>(&self, role: T) -> Result<CreateRoleRes, BoxedError>
    where
        T: Into<CreateRoleReq> + fmt::Debug + Send,
    {
        let role = role.into();

        let resp: Response<CreateRoleRes> = self
            .post("/roles", role)
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
impl CreateRoleTrait for ClientMock {
    async fn create_role<T>(&self, role: T) -> Result<CreateRoleRes, BoxedError>
    where
        T: Into<CreateRoleReq> + fmt::Debug + Send,
    {
        let Some(func) = self.create_role_fn.clone() else {
            panic!("create_role not defined for mock client");
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
    async fn create_role_route_contract() {
        contract(
            "POST",
            "/api/v1/roles",
            ("role", "create_role"),
            json!({ "role": role() }),
            |client| {
                async move {
                    client
                        .create_role(CreateRoleReq {
                            role: CreateRole {
                                name: "admin".to_string(),
                            },
                        })
                        .await
                }
            },
        )
        .await;
    }
}
