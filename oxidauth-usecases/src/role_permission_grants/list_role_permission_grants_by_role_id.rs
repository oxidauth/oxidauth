use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    role_permission_grants::list_role_permission_grants_by_role_id::*,
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
impl<'a, T> Service<&'a ListRolePermissionGrantsByRoleId>
    for ListRolePermissionGrantsByRoleIdUseCase<T>
where
    T: SelectRolePermissionGrantsByRoleIdQuery,
{
    type Response = Vec<RolePermission>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "list_role_permission_grants_by_role_id_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a ListRolePermissionGrantsByRoleId,
    ) -> Result<Self::Response, Self::Error> {
        self.role_permission_grants
            .call(req)
            .await
    }
}
