use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::roles::create_role_role_grant::{
    CreateRoleRoleGrantReq,
    CreateRoleRoleGrantRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RoleRoleGrant;
const METHOD: &str = "create_role_role_grant";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CreateRoleRoleGrantTrait {
    async fn create_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<CreateRoleRoleGrantRes, BoxedError>
    where
        T: Into<CreateRoleRoleGrantReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateRoleRoleGrantTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<CreateRoleRoleGrantRes, BoxedError>
    where
        T: Into<CreateRoleRoleGrantReq> + fmt::Debug + Send,
    {
        let role_role_grant = role_role_grant.into();

        let resp: Response<CreateRoleRoleGrantRes> = self
            .post(
                &format!(
                    "/roles/{}/roles/{}",
                    role_role_grant.parent_id, role_role_grant.child_id
                ),
                None::<CreateRoleRoleGrantReq>,
            )
            .await?;

        let role_role_grant_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_role_grant_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateRoleRoleGrantTrait for ClientMock {
    async fn create_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<CreateRoleRoleGrantRes, BoxedError>
    where
        T: Into<CreateRoleRoleGrantReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_role_role_grant_fn
            .clone()
        else {
            panic!("create_role_role_grant not defined for mock client");
        };

        return func(role_role_grant.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, role, role_role_grant};

    #[tokio::test]
    async fn create_role_role_grant_route_contract() {
        let parent_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();

        contract(
            "POST",
            &format!("/api/v1/roles/{parent_id}/roles/{child_id}"),
            ("role_role_grant", "create_role_role_grant"),
            json!({ "child": role(), "grant": role_role_grant() }),
            move |client| {
                async move {
                    client
                        .create_role_role_grant(CreateRoleRoleGrantReq {
                            parent_id,
                            child_id,
                        })
                        .await
                }
            },
        )
        .await;
    }
}
