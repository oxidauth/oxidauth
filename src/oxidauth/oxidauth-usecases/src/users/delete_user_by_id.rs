use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, service::Service, users::delete_user_by_id::*,
};
use oxidauth_repository::users::delete_user_by_id_query::DeleteUserByIdQuery;

pub struct DeleteUserByIdUseCase<T>
where
    T: DeleteUserByIdQuery,
{
    users: T,
}

impl<T> DeleteUserByIdUseCase<T>
where
    T: DeleteUserByIdQuery,
{
    pub fn new(users: T) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<'a, T> Service<&'a DeleteUserById> for DeleteUserByIdUseCase<T>
where
    T: DeleteUserByIdQuery,
{
    type Response = User;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_user_by_id_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a DeleteUserById,
    ) -> Result<Self::Response, Self::Error> {
        self.users.call(req).await
    }
}
