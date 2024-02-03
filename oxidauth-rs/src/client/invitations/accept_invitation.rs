pub use oxidauth_http::server::api::v1::invitations::accept_invitation::{
    AcceptInvitationParams, AcceptInvitationRes,
};
use oxidauth_http::{
    response::Response,
    server::api::v1::invitations::accept_invitation::AcceptInvitationBodyReq,
};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::users::create_user::CreateUser;

use crate::{client::handle_response, Client, Resource};

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "create_invitaion";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn accept_invitation<T>(
        &self,
        params: T,
    ) -> Result<AcceptInvitationRes, BoxedError>
    where
        T: Into<AcceptInvitationParams> + fmt::Debug,
    {
        let AcceptInvitationParams {
            invitation_id,
            user,
            user_authority,
        } = params.into();

        let body = AcceptInvitationBodyReq {
            user,
            user_authority,
        };

        let resp: Response<AcceptInvitationRes> = self
            .post(
                &format!(
                    "/invitations/{}",
                    invitation_id
                ),
                body,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
