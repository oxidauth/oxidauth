use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    permissions::create_permission::{CreatePermission, CreatePermissionTrait, Permission},
};
use oxidauth_repository::permissions::insert_permission::InsertPermissionQuery;

pub struct CreatePermissionUseCase<T>
where
    T: InsertPermissionQuery,
{
    permissions: T,
}

impl<T> CreatePermissionUseCase<T>
where
    T: InsertPermissionQuery,
{
    pub fn new(permissions: T) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl<T> CreatePermissionTrait for CreatePermissionUseCase<T>
where
    T: InsertPermissionQuery,
{
    #[tracing::instrument(name = "create_permission_usecase", skip(self))]
    async fn create_permission(
        &self,
        req: &CreatePermission,
    ) -> Result<Permission, BoxedError> {
        self.permissions
            .call(req)
            .await
    }
}
