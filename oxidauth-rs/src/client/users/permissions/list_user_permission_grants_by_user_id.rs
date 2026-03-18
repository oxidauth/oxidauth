use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::users::permissions::list_user_permissions_by_user_id::{ListUserPermissionGrantsByUserIdReq, ListUserPermissionGrantsByUserIdRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserPermissionGrant;
const METHOD: &str = "list_user_permission_grants_by_user_id";

#[async_trait]
pub trait ListUserPermissionGrantsByUserIdTrait {
    async fn list_user_permission_grants_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserPermissionGrantsByUserIdRes, BoxedError>
    where
        T: Into<ListUserPermissionGrantsByUserIdReq> + fmt::Debug + Send;
}

#[async_trait]
impl ListUserPermissionGrantsByUserIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_user_permission_grants_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserPermissionGrantsByUserIdRes, BoxedError>
    where
        T: Into<ListUserPermissionGrantsByUserIdReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListUserPermissionGrantsByUserIdRes> = self
            .get(
                &format!(
                    "/users/{}/permissions",
                    params.user_id
                ),
                None::<ListUserPermissionGrantsByUserIdReq>,
            )
            .await?;

        let user_permission_grants_res =
            handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_permission_grants_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ListUserPermissionGrantsByUserIdTrait for ClientMock {
    async fn list_user_permission_grants_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserPermissionGrantsByUserIdRes, BoxedError>
    where
        T: Into<ListUserPermissionGrantsByUserIdReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .list_user_permission_grants_by_user_id_fn
            .clone()
        else {
            panic!(
                "list_user_permission_grants_by_user_id not defined for mock client"
            );
        };

        return func(params.into());
    }
}
