use async_trait::async_trait;

use oxidauth_kernel::{error::BoxedError, roles::update_role::*};
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
impl<'a, T> Service<&'a UpdateRole> for UpdateRoleUseCase<T>
where
    T: UpdateRoleQuery,
{
    type Response = Role;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_role_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a UpdateRole,
    ) -> Result<Self::Response, Self::Error> {
        self.roles.call(req).await
    }
}
