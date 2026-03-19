use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    role_permission_grants::list_role_permission_grants_by_role_id::{
        ListRolePermissionGrantsByRoleId,
        ListRolePermissionGrantsByRoleIdTrait, RolePermission,
    },
};
use oxidauth_repository::role_permission_grants::select_role_permission_grants_by_role_id::SelectRolePermissionGrantsByRoleIdQuery;

pub struct ListRolePermissionGrantsByRoleIdUseCase<T>
where
    T: SelectRolePermissionGrantsByRoleIdQuery,
{
    role_permission_grants: T,
}

impl<T> ListRolePermissionGrantsByRoleIdUseCase<T>
where
    T: SelectRolePermissionGrantsByRoleIdQuery,
{
    pub fn new(role_permission_grants: T) -> Self {
        Self {
            role_permission_grants,
        }
    }
}

#[async_trait]
impl<T> ListRolePermissionGrantsByRoleIdTrait
    for ListRolePermissionGrantsByRoleIdUseCase<T>
where
    T: SelectRolePermissionGrantsByRoleIdQuery,
{
    #[tracing::instrument(
        name = "list_role_permission_grants_by_role_id_usecase",
        skip(self)
    )]
    async fn list_role_permission_grants_by_role_id(
        &self,
        params: &ListRolePermissionGrantsByRoleId,
    ) -> Result<Vec<RolePermission>, BoxedError> {
        self.role_permission_grants
            .call(params)
            .await
    }
}
