use async_trait::async_trait;
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

#[async_trait]
impl<U> CreateUserTrait for CreateUserUseCase<U>
where
    U: InsertUserRepo,
{
    async fn create_user(
        &self,
        params: &UserCreate,
    ) -> Result<User, CreateUserError> {
        self.users
            .call(params)
            .await
            .map_err(|err| {
                dbg!(&err);

                CreateUserError {}
            })
    }
}
