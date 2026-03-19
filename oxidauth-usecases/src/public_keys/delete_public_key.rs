use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    public_keys::{delete_public_key::{DeletePublicKey, DeletePublicKeyTrait}, PublicKey},
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
impl<T> DeletePublicKeyTrait for DeletePublicKeyUseCase<T>
where
    T: DeletePublicKeyQuery,
{
    #[tracing::instrument(name = "delete_public_key_usecase", skip(self))]
    async fn delete_public_key(
        &self,
        params: &DeletePublicKey,
    ) -> Result<PublicKey, BoxedError> {
        self.public_keys
            .call(params)
            .await
    }
}
