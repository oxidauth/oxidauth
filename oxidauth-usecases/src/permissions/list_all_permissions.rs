use async_trait::async_trait;

use oxidauth_kernel::{permissions::list_all_permissions::*, error::BoxedError};
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
impl<'a, T> Service<&'a ListAllPermissions> for ListAllPermissionsUseCase<T>
where
    T: SelectAllPermissionsQuery,
{
    type Response = Vec<Permission>;
    type Error = BoxedError;

    #[tracing::instrument(name = "list_all_permissions_usecase", skip(self))]
    async fn call(&self, req: &'a ListAllPermissions) -> Result<Self::Response, Self::Error> {
        self.permissions
            .call(req)
            .await
    }
}

