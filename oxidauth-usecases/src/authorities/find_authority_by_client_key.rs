use async_trait::async_trait;

use oxidauth_kernel::{
    authorities::{AuthorityNotFoundError, find_authority_by_client_key::*},
    error::BoxedError,
};
use oxidauth_repository::authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery;

pub struct FindAuthorityByClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    authorities: T,
}

impl<T> FindAuthorityByClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindAuthorityByClientKey>
    for FindAuthorityByClientKeyUseCase<T>
where
    T: SelectAuthorityByClientKeyQuery,
{
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "find_authority_by_client_key_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        params: &'a FindAuthorityByClientKey,
    ) -> Result<Self::Response, Self::Error> {
        let authority = self
            .authorities
            .call(params)
            .await?
            .ok_or_else(|| {
                AuthorityNotFoundError::client_key(params.client_key)
            })?;

        Ok(authority)
    }
}
