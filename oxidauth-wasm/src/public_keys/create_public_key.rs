use crate::response::Response;
use async_trait::async_trait;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "create_public_key";

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePublicKeyRes {
    pub public_key: PublicKey,
}

#[async_trait(?Send)]
#[async_trait]
pub trait CreatePublicKeyTrait {
    async fn create_public_key(&self) -> Result<CreatePublicKeyRes, String>;
}

#[async_trait(?Send)]
#[async_trait]
impl CreatePublicKeyTrait for Client {
    async fn create_public_key(&self) -> Result<CreatePublicKeyRes, String> {
        let result = self
            .post("/public_keys", None::<()>)
            .await;

        let Ok(resp) = result else {
            return Err("create public key call failed".to_string());
        };

        let Ok(public_key_res) = handle_response(RESOURCE, METHOD, resp) else {
            return Err("create public key call failed".to_string());
        };

        Ok(public_key_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl CreatePublicKeyTrait for ClientMock {
    async fn create_public_key(&self) -> Result<CreatePublicKeyRes, BoxedError> {
        let Some(func) = self
            .create_public_key_fn
            .clone()
        else {
            panic!("create_public_key not defined for mock client");
        };

        return func();
    }
}
