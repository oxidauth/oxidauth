use async_trait::async_trait;

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
        self.public_keys
            .call(req)
            .await
    }
}
