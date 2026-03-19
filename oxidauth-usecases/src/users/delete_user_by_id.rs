use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    users::delete_user_by_id::{DeleteUserById, DeleteUserByIdTrait, User},
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
impl<T> DeleteUserByIdTrait for DeleteUserByIdUseCase<T>
where
    T: DeleteUserByIdQuery,
{
    #[tracing::instrument(name = "delete_user_by_id_usecase", skip(self))]
    async fn delete_user_by_id(
        &self,
        params: &DeleteUserById,
    ) -> Result<User, BoxedError> {
        self.users.call(params).await
    }
}
