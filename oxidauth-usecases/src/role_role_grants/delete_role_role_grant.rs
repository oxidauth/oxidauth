use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    role_role_grants::delete_role_role_grant::{
        DeleteRoleRoleGrant, DeleteRoleRoleGrantTrait, RoleRoleGrant,
    },
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
impl<T> DeleteRoleRoleGrantTrait for DeleteRoleRoleGrantUseCase<T>
where
    T: DeleteRoleRoleGrantQuery,
{
    #[tracing::instrument(name = "delete_role_role_grant_usecase", skip(self))]
    async fn delete_role_role_grant(
        &self,
        params: &DeleteRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError> {
        self.role_role_grants
            .call(params)
            .await
    }
}
