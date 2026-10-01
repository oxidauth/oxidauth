use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::permissions::delete_role_permission_grant::{
    DeleteRolePermissionGrantReq,
    DeleteRolePermissionGrantRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RolePermissionGrant;
const METHOD: &str = "delete_role_permission_grant";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait DeleteRolePermissionGrantTrait {
    async fn delete_role_permission_grant<T>(
        &self,
        role_permission_grant: T,
    ) -> Result<DeleteRolePermissionGrantRes, BoxedError>
    where
        T: Into<DeleteRolePermissionGrantReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DeleteRolePermissionGrantTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_role_permission_grant<T>(
        &self,
        role_permission_grant: T,
    ) -> Result<DeleteRolePermissionGrantRes, BoxedError>
    where
        T: Into<DeleteRolePermissionGrantReq> + fmt::Debug + Send,
    {
        let role_permission_grant = role_permission_grant.into();

        let resp: Response<DeleteRolePermissionGrantRes> = self
            .delete(
                &format!(
                    "/roles/{}/permissions/{}",
                    role_permission_grant.role_id, role_permission_grant.permission
                ),
                None::<DeleteRolePermissionGrantReq>,
            )
            .await?;

        let role_permission_grant_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_permission_grant_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DeleteRolePermissionGrantTrait for ClientMock {
    async fn delete_role_permission_grant<T>(
        &self,
        role_permission_grant: T,
    ) -> Result<DeleteRolePermissionGrantRes, BoxedError>
    where
        T: Into<DeleteRolePermissionGrantReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_role_permission_grant_fn
            .clone()
        else {
            panic!("delete_role_permission_grant not defined for mock client");
        };

        return func(role_permission_grant.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, role_permission};

    #[tokio::test]
    async fn delete_role_permission_grant_route_contract() {
        let role_id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/roles/{role_id}/permissions/oxidauth:users:read"),
            ("role_permission_grant", "delete_role_permission_grant"),
            role_permission(),
            move |client| {
                async move {
                    client
                        .delete_role_permission_grant(DeleteRolePermissionGrantReq {
                            role_id,
                            permission: "oxidauth:users:read".to_string(),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
