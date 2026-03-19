use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    user_authorities::update_user_authority::{
        UpdateUserAuthority, UpdateUserAuthorityTrait, UserAuthority,
    },
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
impl<T> UpdateUserAuthorityTrait for UpdateUserAuthorityUseCase<T>
where
    T: UpdateUserAuthorityQuery,
{
    #[tracing::instrument(name = "update_user_authority_usecase", skip(self))]
    async fn update_user_authority(
        &self,
        params: &UpdateUserAuthority,
    ) -> Result<UserAuthority, BoxedError> {
        self.user_authorities
            .call(params)
            .await
    }
}
