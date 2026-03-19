use async_trait::async_trait;

use oxidauth_kernel::error::BoxedError;
use oxidauth_kernel::roles::find_role_by_id::{FindRoleById, FindRoleByIdTrait, Role};
use oxidauth_repository::roles::select_role_by_id::SelectRoleByIdQuery;

pub struct FindRoleByIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    roles: T,
}

impl<T> FindRoleByIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> FindRoleByIdTrait for FindRoleByIdUseCase<T>
where
    T: SelectRoleByIdQuery,
{
    #[tracing::instrument(name = "find_role_by_id_usecase", skip(self))]
    async fn find_role_by_id(
        &self,
        req: &FindRoleById,
    ) -> Result<Role, BoxedError> {
        self.roles.call(req).await
    }
}
