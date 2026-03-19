use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    permissions::list_all_permissions::{ListAllPermissions, ListAllPermissionsTrait, Permission},
};
use oxidauth_repository::permissions::select_all_permissions::SelectAllPermissionsQuery;

pub struct ListAllPermissionsUseCase<T>
where
    T: SelectAllPermissionsQuery,
{
    permissions: T,
}

impl<T> ListAllPermissionsUseCase<T>
where
    T: SelectAllPermissionsQuery,
{
    pub fn new(permissions: T) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl<T> ListAllPermissionsTrait for ListAllPermissionsUseCase<T>
where
    T: SelectAllPermissionsQuery,
{
    #[tracing::instrument(name = "list_all_permissions_usecase", skip(self))]
    async fn list_all_permissions(
        &self,
        req: &ListAllPermissions,
    ) -> Result<Vec<Permission>, BoxedError> {
        self.permissions
            .call(req)
            .await
    }
}
