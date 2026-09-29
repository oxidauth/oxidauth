use async_trait::async_trait;
use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::invitations::find_invitation::{
    FindInvitationReq,
    FindInvitationRes,
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
pub trait FindInvitationTrait {
    async fn find_invitation<T>(
        &self,
        params: T,
    ) -> Result<FindInvitationRes, BoxedError>
    where
        T: Into<FindInvitationReq> + fmt::Debug + Send;
}

#[async_trait]
impl FindInvitationTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_invitation<T>(
        &self,
        params: T,
    ) -> Result<FindInvitationRes, BoxedError>
    where
        T: Into<FindInvitationReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<FindInvitationRes> = self
            .get(
                &format!(
                    "/invitations/{}",
                    params.invitation_id
                ),
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
    async fn find_invitation<T>(
        &self,
        params: T,
    ) -> Result<FindInvitationRes, BoxedError>
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
