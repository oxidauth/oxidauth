use async_trait::async_trait;
use uuid::Uuid;

use super::*;

const RESOURCE: Resource = Resource::PublicKey;
const METHOD: &str = "find_public_key_by_id";

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPublicKeyByIdRes {
    pub public_key: PublicKey,
}

#[async_trait(?Send)]
#[async_trait]
pub trait FindPublicKeyByIdTrait {
    async fn find_public_key_by_id<T>(
        &self,
        public_key_id: T,
    ) -> Result<FindPublicKeyByIdRes, String>
    where
        T: Into<Uuid> + fmt::Debug;
}

#[async_trait(?Send)]
#[async_trait]
impl FindPublicKeyByIdTrait for Client {
    async fn find_public_key_by_id<T>(
        &self,
        public_key_id: T,
    ) -> Result<FindPublicKeyByIdRes, String>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let public_key_id = public_key_id.into();

        let result = self
            .get(&format!("/public_keys/{}", public_key_id), None::<Uuid>)
            .await;

        let Ok(resp) = result else {
            return Err("find_public_key_by_id call failed".to_string());
        };

        let public_key_res = handle_response(RESOURCE, METHOD, resp);

        let Ok(public_key_res) = public_key_res else {
            return Err("find_public_key_by_id call failed".to_string());
        };

        Ok(public_key_res)
    }
}

#[cfg(feature = "mock")]
use crate::mock::ClientMock;
use crate::response::Response;

#[cfg(feature = "mock")]
#[async_trait]
impl FindPublicKeyByIdTrait for ClientMock {
    async fn find_public_key_by_id<T>(
        &self,
        public_key_id: T,
    ) -> Result<FindPublicKeyByIdRes, String>
    where
        T: Into<Uuid> + fmt::Debug,
    {
        let Some(func) = self
            .find_public_key_by_id_fn
            .clone()
        else {
            panic!("find_public_key_by_id not defined for mock client");
        };

        return func(public_key_id.into());
    }
}
