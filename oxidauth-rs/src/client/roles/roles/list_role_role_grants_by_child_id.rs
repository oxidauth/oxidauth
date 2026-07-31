use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::roles::roles::list_role_role_grants_by_child_id::{
    ListRoleRoleGrantsByChildIdReq, ListRoleRoleGrantsByChildIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::RoleRoleGrant;
const METHOD: &str = "list_role_role_grants_by_child_id";

#[async_trait]
pub trait ListRoleRoleGrantsByChildIdTrait {
    async fn list_role_role_grants_by_child_id<T>(
        &self,
        params: T,
    ) -> Result<ListRoleRoleGrantsByChildIdRes, BoxedError>
    where
        T: Into<ListRoleRoleGrantsByChildIdReq> + fmt::Debug + Send;
}

#[async_trait]
impl ListRoleRoleGrantsByChildIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_role_role_grants_by_child_id<T>(
        &self,
        params: T,
    ) -> Result<ListRoleRoleGrantsByChildIdRes, BoxedError>
    where
        T: Into<ListRoleRoleGrantsByChildIdReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListRoleRoleGrantsByChildIdRes> = self
            .get(
                &format!(
                    "/roles/{}/roles",
                    params.child_id
                ),
                None::<ListRoleRoleGrantsByChildIdReq>,
            )
            .await?;

        let role_role_grants_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_role_grants_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ListRoleRoleGrantsByChildIdTrait for ClientMock {
    async fn list_role_role_grants_by_child_id<T>(
        &self,
        params: T,
    ) -> Result<ListRoleRoleGrantsByChildIdRes, BoxedError>
    where
        T: Into<ListRoleRoleGrantsByChildIdReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .list_role_role_grants_by_child_id_fn
            .clone()
        else {
            panic!(
                "list_role_role_grants_by_child_id not defined for mock client"
            );
        };

        return func(params.into());
    }
}
