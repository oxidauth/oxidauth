use async_trait::async_trait;

use oxidauth_kernel::{permissions::create_permission::*, error::BoxedError};
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
impl<'a, T> Service<&'a CreatePermission> for CreatePermissionUseCase<T>
where
    T: InsertPermissionQuery,
{
    type Response = Permission;
    type Error = BoxedError;

    #[tracing::instrument(name = "create_permission_usecase", skip(self))]
    async fn call(&self, req: &'a CreatePermission) -> Result<Self::Response, Self::Error> {
        self.permissions
            .call(req)
            .await
    }
}

