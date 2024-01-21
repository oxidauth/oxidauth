use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    public_keys::{create_public_key::CreatePublicKey, PublicKey},
    service::Service,
};
use oxidauth_repository::public_keys::insert_public_key::{
    InsertPublicKeyParams, InsertPublicKeyQuery,
};

use oxidauth_kernel::rsa::{Base64KeyPair, KeyPair};

pub struct CreatePublicKeyUseCase<T>
where
    T: InsertPublicKeyQuery,
{
    public_keys: T,
}

impl<T> CreatePublicKeyUseCase<T>
where
    T: InsertPublicKeyQuery,
{
    pub fn new(public_keys: T) -> Self {
        Self { public_keys }
    }
}

#[async_trait]
impl<'a, T> Service<&'a CreatePublicKey> for CreatePublicKeyUseCase<T>
where
    T: InsertPublicKeyQuery,
{
    type Response = PublicKey;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_public_key_usecase", skip(self))]
    async fn call(
        &self,
        _params: &'a CreatePublicKey,
    ) -> Result<Self::Response, Self::Error> {
        let Base64KeyPair { public, private } = KeyPair::new()?.base64_encode();

        let params = InsertPublicKeyParams {
            id: None,
            public_key: public,
            private_key: private,
        };

        self.public_keys
            .call(&params)
            .await
    }
}
