use crate::response::Response;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::*;

const RESOURCE: Resource = Resource::Permission;
const METHOD: &str = "can";

#[derive(Debug, Serialize, Deserialize)]
pub struct CanReq {
    pub permission: String,
}

#[async_trait(?Send)]
#[async_trait]
pub trait CanTrait {
    async fn can<T>(&self, params: T) -> Result<bool, String>
    where
        T: Into<CanReq> + fmt::Debug;
}

#[async_trait(?Send)]
#[async_trait]
impl CanTrait for Client {
    async fn can<T>(&self, params: T) -> Result<bool, String>
    where
        T: Into<CanReq> + fmt::Debug,
    {
        let params = params.into();

        // let endpoint_string = format!("/can/{}", params.permission);

        // let result = self
        //     .get(&endpoint_string, None::<()>)
        //     .await;

        let Ok(resp): Result<Response<bool>, _> = self
            .get(&format!("/can/{}", params.permission), None::<CanReq>)
            .await
        else {
            return Err("can call failed".to_string());
        };

        let can_res = handle_response(RESOURCE, METHOD, resp);

        let Ok(res) = can_res else {
            return Err("can call failed".to_string());
        };

        Ok(res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl CanTrait for ClientMock {
    async fn can<T>(&self, params: T) -> Result<bool, String>
    where
        T: Into<CanReq> + fmt::Debug + Send,
    {
        let Some(func) = self.can_fn.clone() else {
            panic!("can not defined for mock client");
        };

        return func(params.into());
    }
}
