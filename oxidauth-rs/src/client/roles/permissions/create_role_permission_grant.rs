use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::permissions::create_role_permission_grant::{
    CreateRolePermissionGrantReq, CreateRolePermissionGrantRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RolePermissionGrant;
const METHOD: &str = "create_role_permission_grant";

#[async_trait]
pub trait CreateRolePermissionGrantTrait {
    async fn create_role_permission_grant<T>(
        &self,
        role_permission_grant: T,
    ) -> Result<CreateRolePermissionGrantRes, BoxedError>
    where
        T: Into<CreateRolePermissionGrantReq> + fmt::Debug + Send;
}

#[async_trait]
impl CreateRolePermissionGrantTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_role_permission_grant<T>(
        &self,
        role_permission_grant: T,
    ) -> Result<CreateRolePermissionGrantRes, BoxedError>
    where
        T: Into<CreateRolePermissionGrantReq> + fmt::Debug + Send,
    {
        let role_permission_grant = role_permission_grant.into();

        let resp: Response<CreateRolePermissionGrantRes> = self
            .post(
                &format!(
                    "/roles/{}/permissions/{}",
                    role_permission_grant.role_id,
                    role_permission_grant.permission
                ),
                None::<CreateRolePermissionGrantReq>,
            )
            .await?;

        let role_permission_grant_res =
            handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_permission_grant_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl CreateRolePermissionGrantTrait for ClientMock {
    async fn create_role_permission_grant<T>(
        &self,
        role_permission_grant: T,
    ) -> Result<CreateRolePermissionGrantRes, BoxedError>
    where
        T: Into<CreateRolePermissionGrantReq> + fmt::Debug + Send,
    {
        let Some(func) = self.create_role_permission_grant_fn.clone() else {
            panic!("create_role_permission_grant not defined for mock client");
        };

        return func(role_permission_grant.into());
    }
}
