use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    permissions::find_permission_by_parts::FindPermissionByParts,
    role_permission_grants::delete_role_permission_grant::*,
    roles::find_role_by_id::FindRoleById,
};
use oxidauth_repository::{
    permissions::select_permission_by_parts::SelectPermissionByPartsQuery,
    role_permission_grants::delete_role_permission_grant::*,
    roles::select_role_by_id::SelectRoleByIdQuery,
};

pub struct DeleteRolePermissionGrantUseCase<T, R, P>
where
    T: DeleteRolePermissionGrantQuery,
    R: SelectRoleByIdQuery,
    P: SelectPermissionByPartsQuery,
{
    role_permission_grants: T,
    roles: R,
    permissions: P,
}

impl<T, R, P> DeleteRolePermissionGrantUseCase<T, R, P>
where
    T: DeleteRolePermissionGrantQuery,
    R: SelectRoleByIdQuery,
    P: SelectPermissionByPartsQuery,
{
    pub fn new(role_permission_grants: T, roles: R, permissions: P) -> Self {
        Self {
            role_permission_grants,
            roles,
            permissions,
        }
    }
}

#[async_trait]
impl<'a, T, R, P> Service<&'a DeleteRolePermissionGrant>
    for DeleteRolePermissionGrantUseCase<T, R, P>
where
    T: DeleteRolePermissionGrantQuery,
    R: SelectRoleByIdQuery,
    P: SelectPermissionByPartsQuery,
{
    type Response = RolePermissionGrantDetail;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "delete_role_permission_grant_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a DeleteRolePermissionGrant,
    ) -> Result<Self::Response, Self::Error> {
        self.roles
            .call(&FindRoleById {
                role_id: req.role_id,
            })
            .await?;

        let permission = self
            .permissions
            .call(&FindPermissionByParts {
                permission: req.permission.to_owned(),
            })
            .await?;

        let grant = self
            .role_permission_grants
            .call(&DeleteRolePermissionGrantParams { role_id: req.role_id, permission_id: permission.id })
            .await?;

        Ok(RolePermissionGrantDetail { permission, grant })
    }
}

