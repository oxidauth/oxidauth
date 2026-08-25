use crate::response::Response;
use async_trait::async_trait;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "list_all_public_keys";

#[async_trait(?Send)]
#[async_trait]
pub trait ListAllPublicKeysTrait {
    async fn list_all_public_keys(&self) -> Result<ListAllPublicKeysRes, String>;
}

#[async_trait(?Send)]
#[async_trait]
impl ListAllPublicKeysTrait for Client {
    async fn list_all_public_keys(&self) -> Result<ListAllPublicKeysRes, String> {
        let result = self
            .get("/public_keys", None::<()>)
            .await;

        let Ok(resp) = result else {
            return Err("list_all_public_keys call failed".to_string());
        };

        let public_key_res = handle_response(RESOURCE, METHOD, resp);

        let Ok(public_key_res) = public_key_res else {
            return Err("list_all_public_keys call failed".to_string());
        };

        Ok(public_key_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl ListAllPublicKeysTrait for ClientMock {
    async fn list_all_public_keys(&self) -> Result<ListAllPublicKeysRes, String> {
        let Some(func) = self
            .list_all_public_keys_fn
            .clone()
        else {
            panic!("list_all_public_keys not defined for mock client");
        };

        return func();
    }
}
