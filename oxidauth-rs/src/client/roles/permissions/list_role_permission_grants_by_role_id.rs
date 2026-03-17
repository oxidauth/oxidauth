use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::permissions::list_role_permission_grants_by_role_id::{ListRolePermissionGrantsByRoleIdReq, ListRolePermissionGrantsByRoleIdRes};
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
                &format!(
                    "/roles/{}/permissions",
                    params.role_id
                ),
                None::<ListRolePermissionGrantsByRoleIdReq>,
            )
            .await?;

        let role_permission_grants_res =
            handle_response(RESOURCE, METHOD, resp)?;

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
        let Some(func) = self.list_role_permission_grants_by_role_id_fn.clone() else {
            panic!("list_role_permission_grants_by_role_id not defined for mock client");
        };

        return func(params.into());
    }
}
