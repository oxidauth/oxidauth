use crate::response::Response;
use async_trait::async_trait;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "delete_public_key";

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePublicKeyRes {
    pub public_key: PublicKey,
}

#[async_trait(?Send)]
#[async_trait]
pub trait DeletePublicKeyTrait {
    async fn delete_public_key<T>(&self, public_key_id: T) -> Result<DeletePublicKeyRes, String>
    where
        T: Into<Uuid> + fmt::Debug;
}

#[async_trait(?Send)]
#[async_trait]
impl DeletePublicKeyTrait for Client {
    async fn delete_public_key<T>(&self, public_key_id: T) -> Result<DeletePublicKeyRes, String>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let public_key_id = public_key_id.into();

        let result = self
            .delete(&format!("/public_keys/{}", public_key_id), None::<Uuid>)
            .await;

        let Ok(resp) = result else {
            return Err("delete public keys call failed".to_string());
        };

        let public_key_res = handle_response(RESOURCE, METHOD, resp);

        let Ok(public_key_res) = public_key_res else {
            return Err("delete public keys call failed".to_string());
        };

        Ok(public_key_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

#[cfg(feature = "mock")]
#[async_trait]
impl DeletePublicKeyTrait for ClientMock {
    async fn delete_public_key<T>(&self, public_key_id: T) -> Result<DeletePublicKeyRes, String>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let Some(func) = self
            .delete_public_key_fn
            .clone()
        else {
            panic!("delete_public_key not defined for mock client");
        };

        return func(public_key_id.into());
    }
}
