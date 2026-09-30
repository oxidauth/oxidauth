use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::invitations::find_invitation::{FindInvitationReq, FindInvitationRes};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::users::create_user::CreateUser;

use super::*;
use crate::{Client, Resource, client::handle_response};

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_invitation";

#[async_trait]
pub trait FindInvitationTrait {
    async fn find_invitation<T>(&self, params: T) -> Result<FindInvitationRes, BoxedError>
    where
        T: Into<FindInvitationReq> + fmt::Debug + Send;
}

#[async_trait]
impl FindInvitationTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_invitation<T>(&self, params: T) -> Result<FindInvitationRes, BoxedError>
    where
        T: Into<FindInvitationReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<FindInvitationRes> = self
            .get(
                &format!("/invitations/{}", params.invitation_id),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FindInvitationTrait for ClientMock {
    async fn find_invitation<T>(&self, params: T) -> Result<FindInvitationRes, BoxedError>
    where
        T: Into<FindInvitationReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_invitation_fn
            .clone()
        else {
            panic!("find_invitation not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, invitation};

    #[tokio::test]
    async fn find_invitation_route_contract() {
        let invitation_id = Uuid::new_v4();

        contract(
            "GET",
            &format!("/api/v1/invitations/{invitation_id}"),
            ("user", "find_invitation"),
            json!({ "invitation": invitation() }),
            move |client| {
                async move {
                    client
                        .find_invitation(FindInvitationReq { invitation_id })
                        .await
                }
            },
        )
        .await;
    }
}
