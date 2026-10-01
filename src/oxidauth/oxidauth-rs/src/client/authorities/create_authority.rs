use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::authorities::create_authority::{CreateAuthorityReq, CreateAuthorityRes};
pub use oxidauth_kernel::{
    authorities::{TotpSettings, create_authority::CreateAuthority},
    error::BoxedError,
};

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "create_authority";

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait CreateAuthorityTrait {
    async fn create_authority<T>(&self, authority: T) -> Result<CreateAuthorityRes, BoxedError>
    where
        T: Into<CreateAuthorityReq> + fmt::Debug + Send;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn create_authority<T>(&self, authority: T) -> Result<CreateAuthorityRes, BoxedError>
    where
        T: Into<CreateAuthorityReq> + fmt::Debug + Send,
    {
        let authority = authority.into();

        let resp: Response<CreateAuthorityRes> = self
            .post("/authorities", authority)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl CreateAuthorityTrait for ClientMock {
    async fn create_authority<T>(&self, authority: T) -> Result<CreateAuthorityRes, BoxedError>
    where
        T: Into<CreateAuthorityReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .create_authority_fn
            .clone()
        else {
            panic!("create_authority not defined for mock client");
        };

        return func(authority.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::time::Duration;

    use oxidauth_kernel::{
        JsonValue,
        authorities::{AuthoritySettings, AuthorityStrategy, NbfOffset},
        jwt::EntitlementsEncoding,
    };
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{authority, contract};

    #[tokio::test]
    async fn create_authority_route_contract() {
        contract(
            "POST",
            "/api/v1/authorities",
            ("authority", "create_authority"),
            json!({ "authority": authority() }),
            |client| {
                async move {
                    client
                        .create_authority(CreateAuthorityReq {
                            authority: CreateAuthority {
                                name: "primary".to_string(),
                                client_key: None,
                                status: None,
                                strategy: AuthorityStrategy::UsernamePassword,
                                settings: AuthoritySettings {
                                    jwt_ttl: Duration::from_secs(86_400),
                                    jwt_nbf_offset: NbfOffset::Disabled,
                                    refresh_token_ttl: Duration::from_secs(2_592_000),
                                    totp: TotpSettings::Disabled,
                                    entitlements_encoding: EntitlementsEncoding::Txt,
                                },
                                params: JsonValue::new(json!({})),
                            },
                        })
                        .await
                }
            },
        )
        .await;
    }
}
