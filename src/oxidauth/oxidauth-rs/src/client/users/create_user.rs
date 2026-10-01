use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::users::create_user::{CreateUserReq, CreateUserRes};
use oxidauth_kernel::error::BoxedError;
pub use oxidauth_kernel::users::UserKind;

use super::*;

const RESOURCE: Resource = Resource::User;
const METHOD: &str = "create_user";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CreateUserTrait {
    async fn create_user<T>(&self, user: T) -> Result<CreateUserRes, BoxedError>
    where
        T: Into<CreateUserReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateUserTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_user<T>(&self, user: T) -> Result<CreateUserRes, BoxedError>
    where
        T: Into<CreateUserReq> + fmt::Debug + Send,
    {
        let user = user.into();

        let resp: Response<CreateUserRes> = self
            .post("/users", user)
            .await?;

        let user_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(user_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateUserTrait for ClientMock {
    async fn create_user<T>(&self, user: T) -> Result<CreateUserRes, BoxedError>
    where
        T: Into<CreateUserReq> + fmt::Debug + Send,
    {
        let Some(func) = self.create_user_fn.clone() else {
            panic!("create_user not defined for mock client");
        };

        return func(user.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use oxidauth_kernel::users::create_user::CreateUser;
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, user};

    #[tokio::test]
    async fn create_user_route_contract() {
        contract(
            "POST",
            "/api/v1/users",
            ("user", "create_user"),
            json!({ "user": user() }),
            |client| {
                async move {
                    client
                        .create_user(CreateUserReq {
                            user: CreateUser {
                                id: None,
                                kind: None,
                                status: None,
                                username: "malreynolds".to_string(),
                                email: None,
                                first_name: None,
                                last_name: None,
                                profile: None,
                            },
                        })
                        .await
                }
            },
        )
        .await;
    }
}
