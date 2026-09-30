use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::settings::fetch_setting::{FetchSettingReq, FetchSettingRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Setting;
const METHOD: &str = "fetch_setting";

#[async_trait]
pub trait FetchSettingTrait {
    async fn fetch_setting<T>(&self, params: T) -> Result<FetchSettingRes, BoxedError>
    where
        T: Into<FetchSettingReq> + fmt::Debug + Send;
}

#[async_trait]
impl FetchSettingTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn fetch_setting<T>(&self, params: T) -> Result<FetchSettingRes, BoxedError>
    where
        T: Into<FetchSettingReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<FetchSettingRes> = self
            .get(
                &format!("/settings/{}", params.key),
                None::<FetchSettingReq>,
            )
            .await?;

        let setting_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(setting_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl FetchSettingTrait for ClientMock {
    async fn fetch_setting<T>(&self, params: T) -> Result<FetchSettingRes, BoxedError>
    where
        T: Into<FetchSettingReq> + fmt::Debug + Send,
    {
        let Some(func) = self.fetch_setting_fn.clone() else {
            panic!("fetch_setting not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, setting};

    #[tokio::test]
    async fn fetch_setting_route_contract() {
        contract(
            "GET",
            "/api/v1/settings/bootstrap.plan",
            ("setting", "fetch_setting"),
            json!({ "setting": setting("bootstrap.plan") }),
            |client| {
                async move {
                    client
                        .fetch_setting(FetchSettingReq {
                            key: "bootstrap.plan".to_string(),
                        })
                        .await
                }
            },
        )
        .await;
    }
}
