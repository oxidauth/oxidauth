use async_trait::async_trait;

use oxidauth_kernel::error::BoxedError;
use oxidauth_kernel::roles::find_role_by_name::{FindRoleByName, FindRoleByNameTrait, Role};
use oxidauth_repository::roles::select_role_by_name::SelectRoleByNameQuery;

pub struct FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    roles: T,
}

impl<T> FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> FindRoleByNameTrait for FindRoleByNameUseCase<T>
where
    T: SelectRoleByNameQuery,
{
    #[tracing::instrument(name = "find_role_by_name_usecase", skip(self))]
    async fn find_role_by_name(
        &self,
        req: &FindRoleByName,
    ) -> Result<Role, BoxedError> {
        self.roles.call(req).await
    }
}
