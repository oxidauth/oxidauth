use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::find_users_by_ids::{FindUsersByIdsReq, FindUsersByIdsRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "find_users_by_ids";

#[async_trait]
pub trait FindUsersByIdsTrait {
    async fn find_users_by_ids<T>(&self, params: T) -> Result<FindUsersByIdsRes, BoxedError>
    where
        T: Into<FindUsersByIdsReq> + fmt::Debug + Send;
}

#[async_trait]
impl FindUsersByIdsTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn find_users_by_ids<T>(&self, params: T) -> Result<FindUsersByIdsRes, BoxedError>
    where
        T: Into<FindUsersByIdsReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<FindUsersByIdsRes> = self
            .post("/users/by_ids", Some(params))
            .await?;

        let users_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(users_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FindUsersByIdsTrait for ClientMock {
    async fn find_users_by_ids<T>(&self, params: T) -> Result<FindUsersByIdsRes, BoxedError>
    where
        T: Into<FindUsersByIdsReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .find_users_by_ids_fn
            .clone()
        else {
            panic!("find_users_by_ids not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user};

    #[tokio::test]
    async fn find_users_by_ids_route_contract() {
        let user_id = Uuid::new_v4();
        let not_found = Uuid::new_v4();

        contract(
            "POST",
            "/api/v1/users/by_ids",
            ("user", "find_users_by_ids"),
            json!({
                "users": [user()],
                "user_ids_not_found": [not_found.to_string()],
            }),
            move |client| {
                async move {
                    client
                        .find_users_by_ids(FindUsersByIdsReq {
                            user_ids: vec![user_id, not_found],
                        })
                        .await
                }
            },
        )
        .await;
    }
}
