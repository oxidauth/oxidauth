use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::create_role::{
    CreateRole, CreateRoleReq, CreateRoleRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "create_role";

#[async_trait]
pub trait CreateRoleTrait {
    async fn create_role<T>(
        &self,
        role: T,
    ) -> Result<CreateRoleRes, BoxedError>
    where
        T: Into<CreateRoleReq> + fmt::Debug + Send;
}

#[async_trait]
impl CreateRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_role<T>(
        &self,
        role: T,
    ) -> Result<CreateRoleRes, BoxedError>
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
#[async_trait]
impl CreateRoleTrait for ClientMock {
    async fn create_role<T>(
        &self,
        role: T,
    ) -> Result<CreateRoleRes, BoxedError>
    where
        T: Into<CreateRoleReq> + fmt::Debug + Send,
    {
        let Some(func) = self.create_role_fn.clone() else {
            panic!("create_role not defined for mock client");
        };

        return func(role.into());
    }
}
