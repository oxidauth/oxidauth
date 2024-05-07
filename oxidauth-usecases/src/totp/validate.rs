use async_trait::async_trait;

use oxidauth_kernel::{error::BoxedError, roles::create_role::*};
use oxidauth_repository::roles::insert_role::InsertRoleQuery;

pub struct CreateRoleUseCase<T>
where
    T: InsertRoleQuery,
{
    roles: T,
}

impl<T> CreateRoleUseCase<T>
where
    T: InsertRoleQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<'a, T> Service<&'a CreateRole> for CreateRoleUseCase<T>
where
    T: InsertRoleQuery,
{
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_role_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a CreateRole,
    ) -> Result<Self::Response, Self::Error> {
        self.roles.call(req).await
    }
}
