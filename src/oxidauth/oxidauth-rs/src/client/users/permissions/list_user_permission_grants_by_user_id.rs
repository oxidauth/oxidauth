use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::permissions::list_user_permissions_by_user_id::{
    ListUserPermissionGrantsByUserIdReq,
    ListUserPermissionGrantsByUserIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserPermissionGrant;
const METHOD: &str = "list_user_permission_grants_by_user_id";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait ListUserPermissionGrantsByUserIdTrait {
    async fn list_user_permission_grants_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserPermissionGrantsByUserIdRes, BoxedError>
    where
        T: Into<ListUserPermissionGrantsByUserIdReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
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
                &format!("/users/{}/permissions", params.user_id),
                None::<ListUserPermissionGrantsByUserIdReq>,
            )
            .await?;

        let user_permission_grants_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_permission_grants_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
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
            panic!("list_user_permission_grants_by_user_id not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_permission};

    #[tokio::test]
    async fn list_user_permission_grants_by_user_id_route_contract() {
        let user_id = Uuid::new_v4();

        contract(
            "GET",
            &format!("/api/v1/users/{user_id}/permissions"),
            (
                "user_permission_grant",
                "list_user_permission_grants_by_user_id",
            ),
            json!({ "user_permission_grants": [user_permission()] }),
            move |client| {
                async move {
                    client
                        .list_user_permission_grants_by_user_id(
                            ListUserPermissionGrantsByUserIdReq { user_id },
                        )
                        .await
                }
            },
        )
        .await;
    }
}
