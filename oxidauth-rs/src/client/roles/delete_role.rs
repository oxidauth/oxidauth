use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::delete_role::DeleteRoleRes;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "delete_role";

#[async_trait]
pub trait DeleteRoleTrait {
    async fn delete_role<T>(
        &self,
        role_id: T,
    ) -> Result<DeleteRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl DeleteRoleTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_role<T>(
        &self,
        role_id: T,
    ) -> Result<DeleteRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let role_id = role_id.into();

        let resp: Response<DeleteRoleRes> = self
            .delete(
                &format!("/roles/{}", role_id),
                None::<Uuid>,
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
impl DeleteRoleTrait for ClientMock {
    async fn delete_role<T>(
        &self,
        role_id: T,
    ) -> Result<DeleteRoleRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self.delete_role_fn.clone() else {
            panic!("delete_role not defined for mock client");
        };

        return func(role_id.into());
    }
}
