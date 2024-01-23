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
        req: &'a mut UpdateUser,
    ) -> Result<Self::Response, Self::Error> {
        let user_id = match req.id {
            Some(id) => id,
            None => return Err("no user_id found".into()),
        };

        let current = self
            .user_by_id
            .call(&FindUserById { user_id })
            .await?;

        if req.email.is_none() {
            req.email = current.email;
        }

        if req.status.is_none() {
            req.status = Some(current.status);
        }

        if req.profile.is_none() {
            req.profile = Some(current.profile);
        }

        self.update_user
            .call(req)
            .await
    }
}
