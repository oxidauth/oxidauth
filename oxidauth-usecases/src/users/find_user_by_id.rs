use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    users::find_user_by_id::{FindUserById, FindUserByIdTrait, User},
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
impl<T> FindUserByIdTrait for FindUserByIdUseCase<T>
where
    T: SelectUserByIdQuery,
{
    #[tracing::instrument(name = "find_user_by_id_usecase", skip(self))]
    async fn find_user_by_id(
        &self,
        params: &FindUserById,
    ) -> Result<User, BoxedError> {
        self.users.call(params).await
    }
}
