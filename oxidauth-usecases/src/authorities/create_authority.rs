use async_trait::async_trait;

use oxidauth_kernel::{authorities::create_authority::*, error::BoxedError};
use oxidauth_repository::authorities::insert_authority::InsertAuthorityQuery;

pub struct CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    authorities: T,
}

impl<T> CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    pub fn new(authorities: T) -> Self {
        Self { authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a CreateAuthority> for CreateAuthorityUseCase<T>
where
    T: InsertAuthorityQuery,
{
    type Response = Authority;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_authority_usecase", skip(self))]
    async fn call(&self, req: &'a CreateAuthority) -> Result<Self::Response, Self::Error> {
        self.authorities
            .call(req)
            .await
    }
}

