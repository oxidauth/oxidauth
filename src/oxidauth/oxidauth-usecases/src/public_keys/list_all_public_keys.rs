use async_trait::async_trait;
use base64::prelude::*;

use oxidauth_kernel::{
    error::BoxedError, public_keys::list_all_public_keys::*, service::Service,
};
use oxidauth_repository::public_keys::select_all_public_keys::SelectAllPublicKeysQuery;

pub struct ListAllPublicKeysUseCase<T>
where
    T: SelectAllPublicKeysQuery,
{
    public_keys: T,
}

impl<T> ListAllPublicKeysUseCase<T>
where
    T: SelectAllPublicKeysQuery,
{
    pub fn new(public_keys: T) -> Self {
        Self { public_keys }
    }
}

#[async_trait]
impl<'a, T> Service<&'a ListAllPublicKeys> for ListAllPublicKeysUseCase<T>
where
    T: SelectAllPublicKeysQuery,
{
    type Response = Vec<PublicKey>;
    type Error = BoxedError;

    #[tracing::instrument(name = "list_all_public_keys_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a ListAllPublicKeys,
    ) -> Result<Self::Response, Self::Error> {
        let public_keys = self
            .public_keys
            .call(req)
            .await?;

        let public_keys = public_keys
            .into_iter()
            .map(|mut pk| {
                let decoded = BASE64_STANDARD
                    .decode(&pk.public_key)
                    .unwrap();

                pk.public_key = String::from_utf8(decoded).unwrap();

                pk
            })
            .collect();

        Ok(public_keys)
    }
}
