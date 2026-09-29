use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    users::{find_user_by_id::FindUserById, update_user::*},
};
use oxidauth_repository::users::{
    select_user_by_id_query::SelectUserByIdQuery, update_user::UpdateUserQuery,
};

pub struct UpdateUserUseCase<S, U>
where
    S: SelectUserByIdQuery,
    U: UpdateUserQuery,
{
    user_by_id: S,
    update_user: U,
}

impl<S, U> UpdateUserUseCase<S, U>
where
    S: SelectUserByIdQuery,
    U: UpdateUserQuery,
{
    pub fn new(user_by_id: S, update_user: U) -> Self {
        Self {
            user_by_id,
            update_user,
        }
    }
}

#[async_trait]
impl<'a, S, U> Service<&'a mut UpdateUser> for UpdateUserUseCase<S, U>
where
    S: SelectUserByIdQuery,
    U: UpdateUserQuery,
{
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_user_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a mut UpdateUser,
    ) -> Result<Self::Response, Self::Error> {
        let current = self
            .user_by_id
            .call(&FindUserById { user_id: params.id })
            .await?;

        if params.username.is_none() {
            params.username = Some(current.username);
        }

        if params.email.is_none() {
            params.email = current.email;
        }

        if params.status.is_none() {
            params.status = Some(current.status);
        }

        if params.profile.is_none() {
            params.profile = Some(current.profile);
        }

        self.update_user
            .call(params)
            .await
    }
}
