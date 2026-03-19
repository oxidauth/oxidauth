use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    permissions::delete_permission::{DeletePermission, DeletePermissionTrait, Permission},
};
use oxidauth_repository::permissions::delete_permission::DeletePermissionQuery;

pub struct DeletePermissionUseCase<T>
where
    T: DeletePermissionQuery,
{
    permissions: T,
}

impl<T> DeletePermissionUseCase<T>
where
    T: DeletePermissionQuery,
{
    pub fn new(permissions: T) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl<T> DeletePermissionTrait for DeletePermissionUseCase<T>
where
    T: DeletePermissionQuery,
{
    #[tracing::instrument(name = "delete_permission_usecase", skip(self))]
    async fn delete_permission(
        &self,
        req: &DeletePermission,
    ) -> Result<Permission, BoxedError> {
        self.permissions
            .call(req)
            .await
    }
}
