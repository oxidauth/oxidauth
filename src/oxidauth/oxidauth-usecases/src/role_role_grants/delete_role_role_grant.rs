use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, role_role_grants::delete_role_role_grant::*,
};
use oxidauth_repository::role_role_grants::delete_role_role_grant::DeleteRoleRoleGrantQuery;

pub struct DeleteRoleRoleGrantUseCase<T>
where
    T: DeleteRoleRoleGrantQuery,
{
    role_role_grants: T,
}

impl<T> DeleteRoleRoleGrantUseCase<T>
where
    T: DeleteRoleRoleGrantQuery,
{
    pub fn new(role_role_grants: T) -> Self {
        Self { role_role_grants }
    }
}

#[async_trait]
impl<'a, T> Service<&'a DeleteRoleRoleGrant> for DeleteRoleRoleGrantUseCase<T>
where
    T: DeleteRoleRoleGrantQuery,
{
    type Response = RoleRoleGrant;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_role_role_grant_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a DeleteRoleRoleGrant,
    ) -> Result<Self::Response, Self::Error> {
        self.role_role_grants
            .call(req)
            .await
    }
}
