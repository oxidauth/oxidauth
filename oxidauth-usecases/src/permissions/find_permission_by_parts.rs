use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    permissions::{
        find_permission_by_parts::{FindPermissionByParts, FindPermissionByPartsTrait, Permission},
        PermissionNotFoundError,
    },
};
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
impl<T> FindPermissionByPartsTrait for FindPermissionByPartsUseCase<T>
where
    T: SelectPermissionByPartsQuery,
{
    #[tracing::instrument(
        name = "find_permission_by_parts_usecase",
        skip(self)
    )]
    async fn find_permission_by_parts(
        &self,
        params: &FindPermissionByParts,
    ) -> Result<Permission, BoxedError> {
        let permission = self
            .permissions
            .call(params)
            .await?;

        match permission {
            Some(permission) => Ok(permission),
            None => Err(PermissionNotFoundError::new(
                &params.permission,
            )),
        }
    }
}
