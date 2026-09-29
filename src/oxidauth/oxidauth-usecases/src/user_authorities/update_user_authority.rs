use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, user_authorities::update_user_authority::*,
};
use oxidauth_repository::user_authorities::update_user_authority::UpdateUserAuthorityQuery;

pub struct UpdateUserAuthorityUseCase<T>
where
    T: UpdateUserAuthorityQuery,
{
    user_authorities: T,
}

impl<T> UpdateUserAuthorityUseCase<T>
where
    T: UpdateUserAuthorityQuery,
{
    pub fn new(user_authorities: T) -> Self {
        Self { user_authorities }
    }
}

#[async_trait]
impl<'a, T> Service<&'a UpdateUserAuthority> for UpdateUserAuthorityUseCase<T>
where
    T: UpdateUserAuthorityQuery,
{
    type Response = UserAuthority;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_user_authority_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a UpdateUserAuthority,
    ) -> Result<Self::Response, Self::Error> {
        self.user_authorities
            .call(req)
            .await
    }
}
