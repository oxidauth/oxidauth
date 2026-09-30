use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::permissions::delete_user_permission::{
    DeleteUserPermissionReq,
    DeleteUserPermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserPermissionGrant;
const METHOD: &str = "delete_user_permission_grant";

#[async_trait]
pub trait DeleteUserPermissionGrantTrait {
    async fn delete_user_permission_grant<T>(
        &self,
        user_permission_grant: T,
    ) -> Result<DeleteUserPermissionRes, BoxedError>
    where
        T: Into<DeleteUserPermissionReq> + fmt::Debug + Send;
}

#[async_trait]
impl DeleteUserPermissionGrantTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_user_permission_grant<T>(
        &self,
        user_permission_grant: T,
    ) -> Result<DeleteUserPermissionRes, BoxedError>
    where
        T: Into<DeleteUserPermissionReq> + fmt::Debug + Send,
    {
        let user_permission_grant = user_permission_grant.into();

        let resp: Response<DeleteUserPermissionRes> = self
            .delete(
                &format!(
                    "/users/{}/permissions/{}",
                    user_permission_grant.user_id, user_permission_grant.permission
                ),
                None::<()>,
            )
            .await?;

        let user_permission_grant_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_permission_grant_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeleteUserPermissionGrantTrait for ClientMock {
    async fn delete_user_permission_grant<T>(
        &self,
        user_permission_grant: T,
    ) -> Result<DeleteUserPermissionRes, BoxedError>
    where
        T: Into<DeleteUserPermissionReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_user_permission_grant_fn
            .clone()
        else {
            panic!("delete_user_permission_grant not defined for mock client");
        };

        return func(user_permission_grant.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_permission};

    #[tokio::test]
    async fn delete_user_permission_grant_route_contract() {
        let user_id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/users/{user_id}/permissions/oxidauth:users:read"),
            ("user_permission_grant", "delete_user_permission_grant"),
            json!({ "user_permission": user_permission() }),
            move |client| {
                async move {
                    client
                        .delete_user_permission_grant(DeleteUserPermissionReq {
                            user_id,
                            permission: "oxidauth:users:read".to_string(),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
