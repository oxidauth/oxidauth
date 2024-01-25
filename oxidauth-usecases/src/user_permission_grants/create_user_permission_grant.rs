use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    permissions::{
        find_permission_by_parts::FindPermissionByParts,
        PermissionNotFoundError,
    },
    user_permission_grants::{create_user_permission_grant::*, UserPermission},
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    permissions::select_permission_by_parts::SelectPermissionByPartsQuery,
    user_permission_grants::insert_user_permission_grant::InsertUserPermissionGrantQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

pub struct CreateUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: InsertUserPermissionGrantQuery,
{
    users: U,
    permissions: P,
    user_permission_grants: UP,
}

impl<U, P, UP> CreateUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: InsertUserPermissionGrantQuery,
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
impl<'a, U, P, UP> Service<&'a CreateUserPermission>
    for CreateUserPermissionGrantUseCase<U, P, UP>
where
    U: SelectUserByIdQuery,
    P: SelectPermissionByPartsQuery,
    UP: InsertUserPermissionGrantQuery,
{
    type Response = UserPermission;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "create_user_permission_grant_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a CreateUserPermission,
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
            .call(&CreateUserPermissionGrant {
                user_id: user.id,
                permission_id: permission.id,
            })
            .await?;

        Ok(UserPermission { permission, grant })
    }
}
