use async_trait::async_trait;
use uuid::Uuid;

use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::find_role_by_id::FindRoleByIdRes;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Role;
const METHOD: &str = "find_role_by_id";

#[async_trait]
pub trait FindRoleByIdTrait {
    async fn find_role_by_id<T>(
        &self,
        role_id: T,
    ) -> Result<FindRoleByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send;
}

#[async_trait]
impl FindRoleByIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_role_by_id<T>(
        &self,
        role_id: T,
    ) -> Result<FindRoleByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let role_id = role_id.into();

        let resp: Response<FindRoleByIdRes> = self
            .get(
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
impl FindRoleByIdTrait for ClientMock {
    async fn find_role_by_id<T>(
        &self,
        role_id: T,
    ) -> Result<FindRoleByIdRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
    {
        let Some(func) = self.find_role_by_id_fn.clone() else {
            panic!("find_role_by_id not defined for mock client");
        };

        return func(role_id.into());
    }
}
