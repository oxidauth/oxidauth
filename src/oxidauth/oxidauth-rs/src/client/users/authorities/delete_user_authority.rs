use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::authorities::delete_user_authority::{
    DeleteUserAuthorityReq,
    DeleteUserAuthorityRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "delete_user_authority";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait DeleteUserAuthorityTrait {
    async fn delete_user_authority<T>(
        &self,
        params: T,
    ) -> Result<DeleteUserAuthorityRes, BoxedError>
    where
        T: Into<DeleteUserAuthorityReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DeleteUserAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn delete_user_authority<T>(
        &self,
        params: T,
    ) -> Result<DeleteUserAuthorityRes, BoxedError>
    where
        T: Into<DeleteUserAuthorityReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<DeleteUserAuthorityRes> = self
            .delete(
                &format!(
                    "/users/{}/authorities/{}",
                    params.user_id, params.authority_id
                ),
                None::<DeleteUserAuthorityReq>,
            )
            .await?;

        let user_authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DeleteUserAuthorityTrait for ClientMock {
    async fn delete_user_authority<T>(
        &self,
        params: T,
    ) -> Result<DeleteUserAuthorityRes, BoxedError>
    where
        T: Into<DeleteUserAuthorityReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .delete_user_authority_fn
            .clone()
        else {
            panic!("delete_user_authority not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_authority};

    #[tokio::test]
    async fn delete_user_authority_route_contract() {
        let user_id = Uuid::new_v4();
        let authority_id = Uuid::new_v4();

        contract(
            "DELETE",
            &format!("/api/v1/users/{user_id}/authorities/{authority_id}"),
            ("user_authority", "delete_user_authority"),
            json!({ "user_authority": user_authority() }),
            move |client| {
                async move {
                    client
                        .delete_user_authority(DeleteUserAuthorityReq {
                            user_id,
                            authority_id,
                        })
                        .await
                }
            },
        )
        .await;
    }
}
