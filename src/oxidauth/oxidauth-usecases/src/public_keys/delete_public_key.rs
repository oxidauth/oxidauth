use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    public_keys::{delete_public_key::DeletePublicKey, PublicKey},
    service::Service,
};
use oxidauth_repository::public_keys::delete_public_key::DeletePublicKeyQuery;

pub struct DeletePublicKeyUseCase<T>
where
    T: DeletePublicKeyQuery,
{
    public_keys: T,
}

impl<T> DeletePublicKeyUseCase<T>
where
    T: DeletePublicKeyQuery,
{
    pub fn new(public_keys: T) -> Self {
        Self { public_keys }
    }
}

#[async_trait]
impl<'a, T> Service<&'a DeletePublicKey> for DeletePublicKeyUseCase<T>
where
    T: DeletePublicKeyQuery,
{
    type Response = PublicKey;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_public_key_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a DeletePublicKey,
    ) -> Result<Self::Response, Self::Error> {
        self.public_keys
            .call(params)
            .await
    }
}
