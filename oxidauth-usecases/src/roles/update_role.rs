use async_trait::async_trait;

use oxidauth_kernel::error::BoxedError;
use oxidauth_kernel::roles::update_role::{UpdateRole, UpdateRoleTrait, Role};
use oxidauth_repository::roles::update_role::UpdateRoleQuery;

pub struct UpdateRoleUseCase<T>
where
    T: UpdateRoleQuery,
{
    roles: T,
}

impl<T> UpdateRoleUseCase<T>
where
    T: UpdateRoleQuery,
{
    pub fn new(roles: T) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl<T> UpdateRoleTrait for UpdateRoleUseCase<T>
where
    T: UpdateRoleQuery,
{
    #[tracing::instrument(name = "update_role_usecase", skip(self))]
    async fn update_role(
        &self,
        req: &UpdateRole,
    ) -> Result<Role, BoxedError> {
        self.roles.call(req).await
    }
}
