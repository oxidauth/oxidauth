use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::roles::create_role_role_grant::{
    CreateRoleRoleGrantReq, CreateRoleRoleGrantRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RoleRoleGrant;
const METHOD: &str = "create_role_role_grant";

#[async_trait]
pub trait CreateRoleRoleGrantTrait {
    async fn create_role_role_grant<T>(
        &self,
        role_role_grant: T,
    ) -> Result<CreateRoleRoleGrantRes, BoxedError>
    where
        T: Into<CreateRoleRoleGrantReq> + fmt::Debug + Send;
}

#[async_trait]
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
#[async_trait]
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
