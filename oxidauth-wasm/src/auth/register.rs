use crate::response::Response;
use async_trait::async_trait;
use log::info;

use super::*;

const RESOURCE: Resource = Resource::Auth;
const METHOD: &str = "register";

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStrategy {
    UsernamePassword,
    SingleUseToken,
    Oauth2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterReq {
    pub client_key: Uuid,
    pub params: serde_json::Value,
}

#[async_trait(?Send)]
#[async_trait]
pub trait RegisterTrait {
    async fn register<T>(&self, params: T) -> Result<RegisterRes, String>
    where
        T: Into<RegisterReq> + fmt::Debug;
}

#[async_trait(?Send)]
#[async_trait]
impl RegisterTrait for Client {
    async fn register<T>(&self, params: T) -> Result<RegisterRes, String>
    where
        T: Into<RegisterReq> + fmt::Debug,
    {
        info!("start register call");

        let params = params.into();

        info!("register params {:#?}", params);

        let result = self
            .post("/auth/register", params)
            .await;

        let Ok(resp) = result else {
            return Err("register call failed".to_string());
        };

        let Ok(register_res) = handle_response(RESOURCE, METHOD, resp) else {
            return Err("register call failed".to_string());
        };

        info!("Register res {:#?}", register_res);
        Ok(register_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl RegisterTrait for ClientMock {
    async fn register<T>(&self, params: T) -> Result<RegisterRes, BoxedError>
    where
        T: Into<RegisterReq> + fmt::Debug,
    {
        let Some(func) = self.register_fn.clone() else {
            panic!("register not defined for mock client");
        };

        return func(params.into());
    }
}
