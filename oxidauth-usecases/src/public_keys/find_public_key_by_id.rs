use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, public_keys::find_public_key_by_id::*,
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
impl<T> FindPublicKeyByIdTrait for FindPublicKeyByIdUseCase<T>
where
    T: SelectPublicKeyByIdQuery,
{
    #[tracing::instrument(name = "find_public_key_by_id_usecase", skip(self))]
    async fn find_public_key_by_id(
        &self,
        req: &FindPublicKeyById,
    ) -> Result<PublicKey, BoxedError> {
        self.public_keys
            .call(req)
            .await
    }
}
