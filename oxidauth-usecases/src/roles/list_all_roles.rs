use async_trait::async_trait;

use oxidauth_kernel::error::BoxedError;
use oxidauth_kernel::roles::list_all_roles::{ListAllRoles, ListAllRolesTrait, Role};
use oxidauth_repository::roles::select_all_roles::SelectAllRolesQuery;

pub struct ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    roles: T,
}

impl<T> ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> ListAllRolesTrait for ListAllRolesUseCase<T>
where
    T: SelectAllRolesQuery,
{
    #[tracing::instrument(name = "list_all_roles_usecase", skip(self))]
    async fn list_all_roles(
        &self,
        req: &ListAllRoles,
    ) -> Result<Vec<Role>, BoxedError> {
        self.roles.call(req).await
    }
}
