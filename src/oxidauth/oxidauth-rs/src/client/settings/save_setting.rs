use async_trait::async_trait;
use oxidauth_http::Response;
pub use oxidauth_http::settings::save_setting::{SaveSettingReq, SaveSettingRes};
use oxidauth_kernel::error::BoxedError;

use super::*;

const RESOURCE: Resource = Resource::Setting;
const METHOD: &str = "save_setting";

#[async_trait]
pub trait SaveSettingTrait {
    async fn save_setting<T>(&self, params: T) -> Result<SaveSettingRes, BoxedError>
    where
        T: Into<SaveSettingReq> + fmt::Debug + Send;
}

#[async_trait]
impl SaveSettingTrait for Client {
    #[tracing::instrument(skip(self))]
    async fn save_setting<T>(&self, params: T) -> Result<SaveSettingRes, BoxedError>
    where
        T: Into<SaveSettingReq> + fmt::Debug + Send,
    {
        let params = params.into();

        let resp: Response<SaveSettingRes> = self
            .post("/settings", params)
            .await?;

        let setting_res = handle_response(RESOURCE, METHOD, resp)?;

        Ok(setting_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl SaveSettingTrait for ClientMock {
    async fn save_setting<T>(&self, params: T) -> Result<SaveSettingRes, BoxedError>
    where
        T: Into<SaveSettingReq> + fmt::Debug + Send,
    {
        let Some(func) = self.save_setting_fn.clone() else {
            panic!("save_setting not defined for mock client");
        };

        return func(params.into());
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use oxidauth_kernel::settings::save_setting::SaveSettingParams;
    use serde_json::json;

    use super::*;
    use crate::client::users::contract::{contract, setting};

    #[tokio::test]
    async fn save_setting_route_contract() {
        contract(
            "POST",
            "/api/v1/settings",
            ("setting", "save_setting"),
            json!({ "setting": setting("plan") }),
            |client| {
                async move {
                    client
                        .save_setting(SaveSettingReq {
                            setting: SaveSettingParams {
                                key: "plan".to_string(),
                                value: json!("free"),
                            },
                        })
                        .await
                }
            },
        )
        .await;
    }
}
