use oxidauth_http::Response;
pub use oxidauth_http::invitations::delete_invitation::{DeleteInvitationReq, DeleteInvitationRes};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::invitations::delete_invitation::DeleteInvitationParams;

use super::*;
use crate::{Client, Resource, client::handle_response};

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "delete_invitation";

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
pub trait DeleteInvitationTrait {
    async fn delete_invitation<T>(&self, params: T) -> Result<DeleteInvitationRes, BoxedError>
    where
        T: Into<DeleteInvitationReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl DeleteInvitationTrait for Client {
    #[cfg_attr(not(target_arch = "wasm32"), tracing::instrument(skip(self)))]
    async fn delete_invitation<T>(&self, params: T) -> Result<DeleteInvitationRes, BoxedError>
    where
        T: Into<DeleteInvitationReq> + fmt::Debug + Send,
    {
        let params = params.into();

        // the api handler extracts BOTH the path id and the json body (the
        // route is /invitations/{invitation_id} and the handler reads
        // `Json<DeleteInvitationReq>`), so the req rides as body.
        let resp: Response<DeleteInvitationRes> = self
            .delete(
                &format!("/invitations/{}", params.invitation.id),
                Some(params),
            )
            .await?;

        let invitation_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(invitation_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl DeleteInvitationTrait for ClientMock {
    async fn delete_invitation<T>(&self, params: T) -> Result<DeleteInvitationRes, BoxedError>
    where
        T: Into<DeleteInvitationReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_invitation_fn
            .clone()
        else {
            panic!("delete_invitation not defined for mock client");
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
    async fn delete_invitation_route_contract() {
        let id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/invitations/{id}"),
            ("user", "delete_invitation"),
            json!({ "invitation": invitation() }),
            move |client| {
                async move {
                    client
                        .delete_invitation(DeleteInvitationReq {
                            invitation: DeleteInvitationParams { id },
                        })
                        .await
                }
            },
        )
        .await;
    }
}
