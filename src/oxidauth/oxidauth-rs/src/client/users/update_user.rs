use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::update_user::{UpdateUserBodyReq, UpdateUserRes};
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "update_user";

#[async_trait]
pub trait UpdateUserTrait {
    async fn update_user<T, U>(&self, user_id: T, user: U) -> Result<UpdateUserRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateUserBodyReq> + fmt::Debug + Send;
}

#[async_trait]
impl UpdateUserTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn update_user<T, U>(&self, user_id: T, user: U) -> Result<UpdateUserRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateUserBodyReq> + fmt::Debug + Send,
    {
        let user_id = user_id.into();
        let user = user.into();

        let resp: Response<UpdateUserRes> = self
            .put(&format!("/users/{}", user_id), user)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl UpdateUserTrait for ClientMock {
    async fn update_user<T, U>(&self, user_id: T, user: U) -> Result<UpdateUserRes, BoxedError>
    where
        T: Into<Uuid> + fmt::Debug + Send,
        U: Into<UpdateUserBodyReq> + fmt::Debug + Send,
    {
        let Some(func) = self.update_user_fn.clone() else {
            panic!("update_user not defined for mock client");
        };

        return func(user_id.into(), user.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use oxidauth_http::users::update_user::UpdateUserUser;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{contract, user};

    #[tokio::test]
    async fn update_user_route_contract() {
        let user_id = Uuid::new_v4();

        contract(
            "PUT",
            &format!("/api/v1/users/{user_id}"),
            ("user", "update_user"),
            json!({ "user": user() }),
            move |client| {
                async move {
                    client
                        .update_user(
                            user_id,
                            UpdateUserBodyReq {
                                user: UpdateUserUser {
                                    username: Some("newname".to_string()),
                                    email: None,
                                    first_name: None,
                                    last_name: None,
                                    status: None,
                                    profile: None,
                                },
                            },
                        )
                        .await
                }
            },
        )
        .await;
    }
}
