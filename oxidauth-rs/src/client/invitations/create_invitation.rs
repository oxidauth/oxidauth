use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::invitations::create_invitaions::{
    CreateInvitationReq, CreateInvitationRes,
};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::users::create_user::CreateUser;

use crate::{client::handle_response, Client, Resource};

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "create_invitaion";

impl Client {
    pub async fn create_invitation<T>(
        &self,
        user: T,
    ) -> Result<CreateInvitationRes, BoxedError>
    where
        T: Into<CreateInvitationReq>,
    {
        let user = user.into();

        let resp: Response<CreateInvitationRes> = self
            .post("/invitations", user)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
