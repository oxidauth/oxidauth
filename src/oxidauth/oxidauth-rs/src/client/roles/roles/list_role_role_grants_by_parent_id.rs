use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::roles::list_role_role_grants_by_parent_id::{
    ListRoleRoleGrantsByParentIdReq,
    ListRoleRoleGrantsByParentIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RoleRoleGrant;
const METHOD: &str = "list_role_role_grants_by_parent_id";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ListRoleRoleGrantsByParentIdTrait {
    async fn list_role_role_grants_by_parent_id<T>(
        &self,
        params: T,
    ) -> Result<ListRoleRoleGrantsByParentIdRes, BoxedError>
    where
        T: Into<ListRoleRoleGrantsByParentIdReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListRoleRoleGrantsByParentIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_role_role_grants_by_parent_id<T>(
        &self,
        params: T,
    ) -> Result<ListRoleRoleGrantsByParentIdRes, BoxedError>
    where
        T: Into<ListRoleRoleGrantsByParentIdReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListRoleRoleGrantsByParentIdRes> = self
            .get(
                &format!("/roles/{}/roles", params.parent_id),
                None::<ListRoleRoleGrantsByParentIdReq>,
            )
            .await?;

        let role_role_grants_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_role_grants_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl ListRoleRoleGrantsByParentIdTrait for ClientMock {
    async fn list_role_role_grants_by_parent_id<T>(
        &self,
        params: T,
    ) -> Result<ListRoleRoleGrantsByParentIdRes, BoxedError>
    where
        T: Into<ListRoleRoleGrantsByParentIdReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .list_role_role_grants_by_parent_id_fn
            .clone()
        else {
            panic!("list_role_role_grants_by_parent_id not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, role_role_grant_detail};

    #[tokio::test]
    async fn list_role_role_grants_by_parent_id_route_contract() {
        let parent_id = Uuid::new_v4();

        contract(
            "GET",
            &format!("/api/v1/roles/{parent_id}/roles"),
            ("role_role_grant", "list_role_role_grants_by_parent_id"),
            json!({ "roles": [role_role_grant_detail()] }),
            move |client| {
                async move {
                    client
                        .list_role_role_grants_by_parent_id(ListRoleRoleGrantsByParentIdReq {
                            parent_id,
                        })
                        .await
                }
            },
        )
        .await;
    }
}
