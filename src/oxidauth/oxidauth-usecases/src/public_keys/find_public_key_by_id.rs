use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, public_keys::find_public_key_by_id::*, service::Service,
};
use oxidauth_repository::public_keys::select_public_key_by_id::SelectPublicKeyByIdQuery;

pub struct FindPublicKeyByIdUseCase<T>
where
    T: SelectPublicKeyByIdQuery,
{
    public_keys: T,
}

impl<T> FindPublicKeyByIdUseCase<T>
where
    T: SelectPublicKeyByIdQuery,
{
    pub fn new(public_keys: T) -> Self {
        Self { public_keys }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindPublicKeyById> for FindPublicKeyByIdUseCase<T>
where
    T: SelectPublicKeyByIdQuery,
{
    type Response = PublicKey;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_public_key_by_id_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a FindPublicKeyById,
    ) -> Result<Self::Response, Self::Error> {
        self.public_keys
            .call(req)
            .await
    }
}
