use async_trait::async_trait;

use oxidauth_kernel::error::BoxedError;
use oxidauth_kernel::roles::create_role::{CreateRole, CreateRoleTrait, Role};
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
impl<T> CreateRoleTrait for CreateRoleUseCase<T>
where
    T: InsertRoleQuery,
{
    #[tracing::instrument(name = "create_role_usecase", skip(self))]
    async fn create_role(
        &self,
        req: &CreateRole,
    ) -> Result<Role, BoxedError> {
        self.roles.call(req).await
    }
}
