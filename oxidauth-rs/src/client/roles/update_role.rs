use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::update_role::{
    UpdateRoleReq,
    UpdateRoleRes,
};
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "update_role";

#[async_trait]
pub trait UpdateRoleTrait {
    async fn update_role<T, U>(
        &self,
        role_id: T,
        role: U,
    ) -> Result<UpdateRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateRoleReq> + fmt::Debug + Send;
}

#[async_trait]
impl UpdateRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn update_role<T, U>(
        &self,
        role_id: T,
        role: U,
    ) -> Result<UpdateRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateRoleReq> + fmt::Debug + Send,
    {
        let role_id = role_id.into();
        let role = role.into();

        let resp: Response<UpdateRoleRes> = self
            .post(
                &format!("/roles/{}", role_id),
                role,
            )
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl UpdateRoleTrait for ClientMock {
    async fn update_role<T, U>(
        &self,
        role_id: T,
        role: U,
    ) -> Result<UpdateRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateRoleReq> + fmt::Debug + Send,
    {
        let Some(func) = self.update_role_fn.clone() else {
            panic!("update_role not defined for mock client");
        };

        return func(role_id.into(), role.into());
    }
}
