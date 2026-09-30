use async_trait::async_trait;
pub use oxidauth_http::invitations::accept_invitation::AcceptInvitationRes;
use oxidauth_http::{Response, invitations::accept_invitation::AcceptInvitationBodyReq};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::{
    auth::{authenticate::AuthenticateParams, register::RegisterParams},
    invitations::accept_invitation::{AcceptInvitationParams, AcceptInvitationUserParams},
    users::create_user::CreateUser,
};
pub use oxidauth_services::auth::strategies::username_password::registrar::UsernamePasswordRegisterParams;

use super::*;
use crate::{Client, Resource, client::handle_response};

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "accept_invitation";

#[async_trait]
pub trait AcceptInvitationTrait {
    async fn accept_invitation<T>(&self, params: T) -> Result<AcceptInvitationRes, BoxedError>
    where
        T: Into<AcceptInvitationParams> + fmt::Debug + Send;
}

#[async_trait]
impl AcceptInvitationTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn accept_invitation<T>(&self, params: T) -> Result<AcceptInvitationRes, BoxedError>
    where
        T: Into<AcceptInvitationParams> + fmt::Debug + Send,
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
            .post(&format!("/invitations/{}", invitation_id), body)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl AcceptInvitationTrait for ClientMock {
    async fn accept_invitation<T>(&self, params: T) -> Result<AcceptInvitationRes, BoxedError>
    where
        T: Into<AcceptInvitationParams> + fmt::Debug + Send,
    {
        let Some(func) = self
            .accept_invitation_fn
            .clone()
        else {
            panic!("accept_invitation not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use oxidauth_kernel::JsonValue;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user};

    #[tokio::test]
    async fn accept_invitation_route_contract() {
        let invitation_id = Uuid::new_v4();

        contract(
            "POST",
            &format!("/api/v1/invitations/{invitation_id}"),
            ("user", "accept_invitation"),
            json!({ "user": user() }),
            move |client| {
                async move {
                    client
                        .accept_invitation(AcceptInvitationParams {
                            invitation_id,
                            user: AcceptInvitationUserParams {
                                username: "invited".to_string(),
                                email: None,
                                first_name: None,
                                last_name: None,
                                profile: None,
                            },
                            user_authority: RegisterParams {
                                client_key: Uuid::new_v4(),
                                params: JsonValue::new(json!({ "password": "hunter2" })),
                            },
                        })
                        .await
                }
            },
        )
        .await;
    }
}
