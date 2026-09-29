use async_trait::async_trait;

use oxidauth_kernel::{error::BoxedError, roles::delete_role::*};
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
impl<'a, T> Service<&'a DeleteRole> for DeleteRoleUseCase<T>
where
    T: DeleteRoleQuery,
{
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_role_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a DeleteRole,
    ) -> Result<Self::Response, Self::Error> {
        self.roles.call(req).await
    }
}
