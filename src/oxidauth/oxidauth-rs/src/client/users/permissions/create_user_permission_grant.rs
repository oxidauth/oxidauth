use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::permissions::create_user_permission::{
    CreateUserPermissionReq,
    CreateUserPermissionRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserPermissionGrant;
const METHOD: &str = "create_user_permission_grant";

#[async_trait]
pub trait CreateUserPermissionGrantTrait {
    async fn create_user_permission_grant<T>(
        &self,
        user_permission_grant: T,
    ) -> Result<CreateUserPermissionRes, BoxedError>
    where
        T: Into<CreateUserPermissionReq> + fmt::Debug + Send;
}

#[async_trait]
impl CreateUserPermissionGrantTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_user_permission_grant<T>(
        &self,
        user_permission_grant: T,
    ) -> Result<CreateUserPermissionRes, BoxedError>
    where
        T: Into<CreateUserPermissionReq> + fmt::Debug + Send,
    {
        let user_permission_grant = user_permission_grant.into();

        let resp: Response<CreateUserPermissionRes> = self
            .post(
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
impl CreateUserPermissionGrantTrait for ClientMock {
    async fn create_user_permission_grant<T>(
        &self,
        user_permission_grant: T,
    ) -> Result<CreateUserPermissionRes, BoxedError>
    where
        T: Into<CreateUserPermissionReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_user_permission_grant_fn
            .clone()
        else {
            panic!("create_user_permission_grant not defined for mock client");
        };

        return func(user_permission_grant.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_permission};

    #[tokio::test]
    async fn create_user_permission_grant_route_contract() {
        let user_id = Uuid::new_v4();

        // the create grant returns the raw UserPermission payload (no wrapper
        // key), unlike its delete sibling — shape pinned here
        contract(
            "POST",
            &format!("/api/v1/users/{user_id}/permissions/oxidauth:users:read"),
            ("user_permission_grant", "create_user_permission_grant"),
            user_permission(),
            move |client| {
                async move {
                    client
                        .create_user_permission_grant(CreateUserPermissionReq {
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
