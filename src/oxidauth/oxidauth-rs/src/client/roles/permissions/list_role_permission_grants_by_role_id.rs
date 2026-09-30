use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::roles::permissions::list_role_permission_grants_by_role_id::{
    ListRolePermissionGrantsByRoleIdReq,
    ListRolePermissionGrantsByRoleIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RolePermissionGrant;
const METHOD: &str = "list_role_permission_grants_by_role_id";

#[async_trait]
pub trait ListRolePermissionGrantsByRoleIdTrait {
    async fn list_role_permission_grants_by_role_id<T>(
        &self,
        params: T,
    ) -> Result<ListRolePermissionGrantsByRoleIdRes, BoxedError>
    where
        T: Into<ListRolePermissionGrantsByRoleIdReq> + fmt::Debug + Send;
}

#[async_trait]
impl ListRolePermissionGrantsByRoleIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_role_permission_grants_by_role_id<T>(
        &self,
        params: T,
    ) -> Result<ListRolePermissionGrantsByRoleIdRes, BoxedError>
    where
        T: Into<ListRolePermissionGrantsByRoleIdReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListRolePermissionGrantsByRoleIdRes> = self
            .get(
                &format!("/roles/{}/permissions", params.role_id),
                None::<ListRolePermissionGrantsByRoleIdReq>,
            )
            .await?;

        let role_permission_grants_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_permission_grants_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ListRolePermissionGrantsByRoleIdTrait for ClientMock {
    async fn list_role_permission_grants_by_role_id<T>(
        &self,
        params: T,
    ) -> Result<ListRolePermissionGrantsByRoleIdRes, BoxedError>
    where
        T: Into<ListRolePermissionGrantsByRoleIdReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .list_role_permission_grants_by_role_id_fn
            .clone()
        else {
            panic!("list_role_permission_grants_by_role_id not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, role_permission};

    #[tokio::test]
    async fn list_role_permission_grants_by_role_id_route_contract() {
        let role_id = Uuid::new_v4();

        contract(
            "GET",
            &format!("/api/v1/roles/{role_id}/permissions"),
            (
                "role_permission_grant",
                "list_role_permission_grants_by_role_id",
            ),
            json!({ "permissions": [role_permission()] }),
            move |client| {
                async move {
                    client
                        .list_role_permission_grants_by_role_id(
                            ListRolePermissionGrantsByRoleIdReq { role_id },
                        )
                        .await
                }
            },
        )
        .await;
    }
}
