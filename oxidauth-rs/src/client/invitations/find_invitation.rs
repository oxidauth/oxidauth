use oxidauth_http::response::Response;
pub use oxidauth_http::server::api::v1::invitations::find_invitation::{
    FindInvitationReq, FindInvitationRes,
};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::users::create_user::CreateUser;

use crate::{client::handle_response, Client, Resource};

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "create_invitaion";

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn find_invitation<T>(
        &self,
        params: T,
    ) -> Result<FindInvitationRes, BoxedError>
    where
        T: Into<FindInvitationReq> + fmt::Debug,
    {
        let params = params.into();

        let resp: Response<FindInvitationRes> = self
            .get(
                &format!("/invitations/{}", params.id),
                None::<()>,
            )
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}
