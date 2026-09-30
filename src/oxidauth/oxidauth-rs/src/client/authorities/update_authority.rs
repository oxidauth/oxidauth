use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::authorities::update_authority::{
    UpdateAuthorityPathReq,
    UpdateAuthorityReq,
    UpdateAuthorityRes,
};
pub use oxidauth_kernel::authorities::update_authority::UpdateAuthority;
use oxidauth_kernel::error::BoxedError;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::Authority;
const METHOD: &str = "update_authority";

#[async_trait]
pub trait UpdateAuthorityTrait {
    async fn update_authority<T, U>(
        &self,
        authority_id: U,
        params: T,
    ) -> Result<UpdateAuthorityRes, BoxedError>
    where
        U: Into<Uuid> + fmt::Debug + Send,
        T: Into<UpdateAuthorityReq> + fmt::Debug + Send;
}

#[async_trait]
impl UpdateAuthorityTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn update_authority<T, U>(
        &self,
        authority_id: U,
        params: T,
    ) -> Result<UpdateAuthorityRes, BoxedError>
    where
        U: Into<Uuid> + fmt::Debug + Send,
        T: Into<UpdateAuthorityReq> + fmt::Debug + Send,
    {
        let authority_id = authority_id.into();
        let params = params.into();

        let resp: Response<UpdateAuthorityRes> = self
            .put(&format!("/authorities/{}", authority_id), params)
            .await?;

        let authority_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(authority_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl UpdateAuthorityTrait for ClientMock {
    async fn update_authority<T, U>(
        &self,
        authority_id: U,
        params: T,
    ) -> Result<UpdateAuthorityRes, BoxedError>
    where
        U: Into<Uuid> + fmt::Debug + Send,
        T: Into<UpdateAuthorityReq> + fmt::Debug + Send,
    {
        let Some(func) = self
            .update_authority_fn
            .clone()
        else {
            panic!("update_authority not defined for mock client");
        };

        return func(authority_id.into(), params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::time::Duration;

    use oxidauth_kernel::authorities::{AuthoritySettings, NbfOffset, TotpSettings};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::client::users::contract::{authority, contract};

    #[tokio::test]
    async fn update_authority_route_contract() {
        let authority_id = Uuid::new_v4();

        contract(
            "PUT",
            &format!("/api/v1/authorities/{authority_id}"),
            ("authority", "update_authority"),
            json!({ "authority": authority() }),
            move |client| {
                async move {
                    client
                        .update_authority(
                            authority_id,
                            UpdateAuthorityReq {
                                authority: UpdateAuthority {
                                    id: None,
                                    name: "primary".to_string(),
                                    client_key: None,
                                    status: None,
                                    strategy:
                                        oxidauth_kernel::authorities::AuthorityStrategy::Oauth2,
                                    settings: AuthoritySettings {
                                        jwt_ttl: Duration::from_secs(3_600),
                                        jwt_nbf_offset: NbfOffset::Disabled,
                                        refresh_token_ttl: Duration::from_secs(2_592_000),
                                        totp: TotpSettings::Disabled,
                                        entitlements_encoding:
                                            oxidauth_kernel::jwt::EntitlementsEncoding::Txt,
                                    },
                                    params: json!({}),
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
