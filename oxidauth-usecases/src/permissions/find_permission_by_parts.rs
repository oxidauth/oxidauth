use async_trait::async_trait;

use oxidauth_kernel::{permissions::find_permission_by_parts::*, error::BoxedError};
use oxidauth_repository::permissions::select_permission_by_parts::SelectPermissionByPartsQuery;

pub struct FindPermissionByPartsUseCase<T>
where
    T: SelectPermissionByPartsQuery,
{
    permissions: T,
}

impl<T> FindPermissionByPartsUseCase<T>
where
    T: SelectPermissionByPartsQuery,
{
    pub fn new(permissions: T) -> Self {
        Self { permissions }
    }
}

#[async_trait]
impl<'a, T> Service<&'a FindPermissionByParts> for FindPermissionByPartsUseCase<T>
where
    T: SelectPermissionByPartsQuery,
{
    type Response = Permission;
    type Error = BoxedError;

    #[tracing::instrument(name = "find_permission_by_parts_usecase", skip(self))]
    async fn call(&self, req: &'a FindPermissionByParts) -> Result<Self::Response, Self::Error> {
        self.permissions
            .call(req)
            .await
    }
}

