use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::auth::register::{RegisterReq, RegisterRes};
pub use oxidauth_kernel::authorities::AuthorityStrategy;
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Auth;
const METHOD: &str = "register";

#[async_trait]
pub trait RegisterTrait {
    async fn register<T>(&self, params: T) -> Result<RegisterRes, BoxedError>
    where
        T: Into<RegisterReq> + fmt::Debug + Send;
}

#[async_trait]
impl RegisterTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn register<T>(&self, params: T) -> Result<RegisterRes, BoxedError>
    where
        T: Into<RegisterReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<RegisterRes> = self
            .post("/auth/register", params)
            .await?;

        let role_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(role_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl RegisterTrait for ClientMock {
    async fn register<T>(&self, params: T) -> Result<RegisterRes, BoxedError>
    where
        T: Into<RegisterReq> + fmt::Debug + Send,
    {
        let Some(func) = self.register_fn.clone() else {
            panic!("register not defined for mock client");
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
    use crate::client::users::contract::{contract, uid};

    #[tokio::test]
    async fn register_route_contract() {
        contract(
            "POST",
            "/api/v1/auth/register",
            ("auth", "register"),
            json!({ "jwt": "header.payload.signature", "refresh_token": uid() }),
            |client| {
                async move {
                    client
                        .register(RegisterReq {
                            client_key: Uuid::new_v4(),
                            params: JsonValue::new(json!({ "password": "hunter2" })),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
