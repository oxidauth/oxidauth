use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::authorities::list_user_authorities_by_user_id::{
    ListUserAuthoritiesByUserIdReq,
    ListUserAuthoritiesByUserIdRes,
};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::UserAuthority;
const METHOD: &str = "list_user_authorities_by_user_id";

#[async_trait]
pub trait ListUserAuthoritiesByUserIdTrait {
    async fn list_user_authorities_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserAuthoritiesByUserIdRes, BoxedError>
    where
        T: Into<ListUserAuthoritiesByUserIdReq> + fmt::Debug + Send;
}

#[async_trait]
impl ListUserAuthoritiesByUserIdTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn list_user_authorities_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserAuthoritiesByUserIdRes, BoxedError>
    where
        T: Into<ListUserAuthoritiesByUserIdReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<ListUserAuthoritiesByUserIdRes> = self
            .get(
                &format!("/users/{}/authorities", params.user_id),
                None::<ListUserAuthoritiesByUserIdReq>,
            )
            .await?;

        let user_authorities_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_authorities_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ListUserAuthoritiesByUserIdTrait for ClientMock {
    async fn list_user_authorities_by_user_id<T>(
        &self,
        params: T,
    ) -> Result<ListUserAuthoritiesByUserIdRes, BoxedError>
    where
        T: Into<ListUserAuthoritiesByUserIdReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .list_user_authorities_by_user_id_fn
            .clone()
        else {
            panic!("list_user_authorities_by_user_id not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user_authority_with_authority};

    #[tokio::test]
    async fn list_user_authorities_by_user_id_route_contract() {
        let user_id = Uuid::new_v4();

        contract(
            "GET",
            &format!("/api/v1/users/{user_id}/authorities"),
            ("user_authority", "list_user_authorities_by_user_id"),
            json!({ "user_authorities": [user_authority_with_authority()] }),
            move |client| {
                async move {
                    client
                        .list_user_authorities_by_user_id(ListUserAuthoritiesByUserIdReq {
                            user_id,
                        })
                        .await
                }
            },
        )
        .await;
    }
}
