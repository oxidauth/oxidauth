use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::roles::delete_role_role_grant::{
    DeleteRoleRoleGrantReq,
    DeleteRoleRoleGrantRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RoleRoleGrant;
const METHOD: &str = "delete_role_role_grant";

#[async_trait]
pub trait DeleteRoleRoleGrantTrait {
    async fn delete_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<DeleteRoleRoleGrantRes, BoxedError>
    where
        T: Into<DeleteRoleRoleGrantReq> + fmt::Debug + Send;
}

#[async_trait]
impl DeleteRoleRoleGrantTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<DeleteRoleRoleGrantRes, BoxedError>
    where
        T: Into<DeleteRoleRoleGrantReq> + fmt::Debug + Send,
    {
        let role_role_grant = role_role_grant.into();

        let resp: Response<DeleteRoleRoleGrantRes> = self
            .delete(
                &format!(
                    "/roles/{}/roles/{}",
                    role_role_grant.parent_id, role_role_grant.child_id
                ),
                None::<DeleteRoleRoleGrantReq>,
            )
            .await?;

        let role_role_grant_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_role_grant_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeleteRoleRoleGrantTrait for ClientMock {
    async fn delete_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<DeleteRoleRoleGrantRes, BoxedError>
    where
        T: Into<DeleteRoleRoleGrantReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_role_role_grant_fn
            .clone()
        else {
            panic!("delete_role_role_grant not defined for mock client");
        };

        return func(role_role_grant.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, role_role_grant};

    #[tokio::test]
    async fn delete_role_role_grant_route_contract() {
        let parent_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/roles/{parent_id}/roles/{child_id}"),
            ("role_role_grant", "delete_role_role_grant"),
            json!({ "grant": role_role_grant() }),
            move |client| {
                async move {
                    client
                        .delete_role_role_grant(DeleteRoleRoleGrantReq {
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
