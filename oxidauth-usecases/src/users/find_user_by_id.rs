use async_trait::async_trait;


use oxidauth_kernel::{
    error::BoxedError, service::Service, users::find_user_by_id::*,
};
use oxidauth_repository::users::select_user_by_id_query::SelectUserByIdQuery;

pub struct FindUserByIdUseCase<T>
where
    T: SelectUserByIdQuery,
{
    users: T,
}

impl<T> FindUserByIdUseCase<T>
where
    T: SelectUserByIdQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindUserById> for FindUserByIdUseCase<T>
where
    T: SelectUserByIdQuery,
{
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_user_by_id_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a FindUserById,
    ) -> Result<Self::Response, Self::Error> {
        self.users.call(req).await
    }
}
