use oxidauth_kernel::{service::Service, users::user_create::*};
use oxidauth_repository::users::insert_user::InsertUserRepo;

pub struct CreateUserUseCase<U>
where
    U: InsertUserRepo,
{
    users: U,
}

impl<U> CreateUserUseCase<U>
where
    U: InsertUserRepo,
{
    pub fn new(users: U) -> Self {
        Self { users }
    }
}

impl<'a, U> Service<&'a UserCreate> for CreateUserUseCase<U>
where
    U: InsertUserRepo,
{
    type Response = User;
    type Error = CreateUserError;

    async fn call(
        &self,
        params: &'a UserCreate,
    ) -> Result<Self::Response, Self::Error> {
        self.users
            .call(params)
            .await
            .map_err(|err| CreateUserError {})
    }
}
