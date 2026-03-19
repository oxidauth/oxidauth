use async_trait::async_trait;

use oxidauth_kernel::error::BoxedError;
use oxidauth_kernel::roles::delete_role::{DeleteRole, DeleteRoleTrait, Role};
use oxidauth_repository::roles::delete_role::DeleteRoleQuery;

pub struct DeleteRoleUseCase<T>
where
    T: DeleteRoleQuery,
{
    roles: T,
}

impl<T> DeleteRoleUseCase<T>
where
    T: DeleteRoleQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> DeleteRoleTrait for DeleteRoleUseCase<T>
where
    T: DeleteRoleQuery,
{
    #[tracing::instrument(name = "delete_role_usecase", skip(self))]
    async fn delete_role(
        &self,
        req: &DeleteRole,
    ) -> Result<Role, BoxedError> {
        self.roles.call(req).await
    }
}
