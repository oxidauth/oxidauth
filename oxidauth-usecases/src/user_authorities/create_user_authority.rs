use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, user_authorities::create_user_authority::*,
};
use oxidauth_repository::user_authorities::insert_user_authority::InsertUserAuthorityQuery;

pub struct CreateUserAuthorityUseCase<T>
where
    T: InsertUserAuthorityQuery,
{
    user_authorities: T,
}

impl<T> CreateUserAuthorityUseCase<T>
where
    T: InsertUserAuthorityQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a CreateUserAuthority> for CreateUserAuthorityUseCase<T>
where
    T: InsertUserAuthorityQuery,
{
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_user_authority_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a CreateUserAuthority,
    ) -> Result<Self::Response, Self::Error> {
        self.user_authorities
            .call(req)
            .await
    }
}
