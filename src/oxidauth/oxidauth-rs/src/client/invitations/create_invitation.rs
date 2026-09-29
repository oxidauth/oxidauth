use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::invitations::create_invitation::{
    CreateInvitationParams,
    CreateInvitationReq,
    CreateInvitationRes,
};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::users::create_user::CreateUser;

use super::*;
use crate::{
    Client,
    Resource,
    client::handle_response,
};

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "create_invitaion";

#[async_trait]
pub trait CreateInvitationTrait {
    async fn create_invitation<T>(
        &self,
        user: T,
    ) -> Result<CreateInvitationRes, BoxedError>
    where
        T: Into<CreateInvitationReq> + fmt::Debug + Send;
}

#[async_trait]
impl CreateInvitationTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_invitation<T>(
        &self,
        user: T,
    ) -> Result<CreateInvitationRes, BoxedError>
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
#[async_trait]
impl CreateInvitationTrait for ClientMock {
    async fn create_invitation<T>(
        &self,
        user: T,
    ) -> Result<CreateInvitationRes, BoxedError>
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
