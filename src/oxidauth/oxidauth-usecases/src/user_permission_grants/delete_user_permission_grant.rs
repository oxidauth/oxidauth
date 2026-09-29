use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    permissions::{
        find_permission_by_parts::FindPermissionByParts,
        PermissionNotFoundError,
    },
    service::Service,
    user_permission_grants::{delete_user_permission_grant::*, UserPermission},
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    permissions::select_permission_by_parts::SelectPermissionByPartsQuery,
    user_permission_grants::delete_user_permission_grant::DeleteUserPermissionGrantQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

pub struct DeleteUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: DeleteUserPermissionGrantQuery,
{
    users: U,
    permissions: P,
    user_permission_grants: UP,
}

impl<U, P, UP> DeleteUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: DeleteUserPermissionGrantQuery,
{
    pub fn new(users: U, permissions: P, user_permission_grants: UP) -> Self {
        Self {
            users,
            permissions,
            user_permission_grants,
        }
    }
}

#[async_trait]
impl<'a, U, P, UP> Service<&'a DeleteUserPermission>
    for DeleteUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: DeleteUserPermissionGrantQuery,
{
    type Response = UserPermission;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "delete_user_permission_grant_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a DeleteUserPermission,
    ) -> Result<Self::Response, Self::Error> {
        let user = self
            .users
            .call(&FindUserById {
                user_id: req.user_id,
            })
            .await?;

        let permission = self
            .permissions
            .call(&FindPermissionByParts {
                permission: req.permission.clone(),
            })
            .await?
            .ok_or_else(|| PermissionNotFoundError::new(&req.permission))?;

        let grant = self
            .user_permission_grants
            .call(&DeleteUserPermissionGrant {
                user_id: user.id,
                permission_id: permission.id,
            })
            .await?;

        Ok(UserPermission { permission, grant })
    }
}
