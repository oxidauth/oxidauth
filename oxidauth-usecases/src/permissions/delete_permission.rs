use async_trait::async_trait;

use oxidauth_kernel::{permissions::delete_permission::*, error::BoxedError};
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
impl<'a, T> Service<&'a DeletePermission> for DeletePermissionUseCase<T>
where
    T: DeletePermissionQuery,
{
    type Response = Permission;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_permission_usecase", skip(self))]
    async fn call(&self, req: &'a DeletePermission) -> Result<Self::Response, Self::Error> {
        self.permissions
            .call(req)
            .await
    }
}


