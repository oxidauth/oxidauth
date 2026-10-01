use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::invitations::create_invitation::{CreateInvitationReq, CreateInvitationRes};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::{
    invitations::create_invitation::CreateInvitationParams,
    users::create_user::CreateUser,
};

use super::*;
use crate::{Client, Resource, client::handle_response};

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "create_invitation";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CreateInvitationTrait {
    async fn create_invitation<T>(&self, user: T) -> Result<CreateInvitationRes, BoxedError>
    where
        T: Into<CreateInvitationReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateInvitationTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_invitation<T>(&self, user: T) -> Result<CreateInvitationRes, BoxedError>
    where
        T: Into<CreateInvitationReq> + fmt::Debug + Send,
    {
        let user = user.into();

        let resp: Response<CreateInvitationRes> = self
            .post("/invitations", user)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateInvitationTrait for ClientMock {
    async fn create_invitation<T>(&self, user: T) -> Result<CreateInvitationRes, BoxedError>
    where
        T: Into<CreateInvitationReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_invitation_fn
            .clone()
        else {
            panic!("create_invitation not defined for mock client");
        };

        return func(user.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use oxidauth_kernel::users::create_user::CreateUser;
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, invitation, user};

    #[tokio::test]
    async fn create_invitation_route_contract() {
        contract(
            "POST",
            "/api/v1/invitations",
            ("user", "create_invitation"),
            json!({ "invitation": invitation(), "user": user() }),
            |client| {
                async move {
                    client
                        .create_invitation(CreateInvitationReq {
                            invitation: CreateInvitationParams {
                                id: None,
                                expires_at: None,
                                user: CreateUser {
                                    id: None,
                                    kind: None,
                                    status: None,
                                    username: "invited".to_string(),
                                    email: None,
                                    first_name: None,
                                    last_name: None,
                                    profile: None,
                                },
                            },
                        })
                        .await
                }
            },
        )
        .await;
    }
}
