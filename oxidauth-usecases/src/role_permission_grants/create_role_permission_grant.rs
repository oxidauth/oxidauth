use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    permissions::{
        find_permission_by_parts::FindPermissionByParts,
        PermissionNotFoundError,
    },
    role_permission_grants::create_role_permission_grant::{
        CreateRolePermissionGrant, CreateRolePermissionGrantTrait,
        RolePermission,
    },
    roles::find_role_by_id::FindRoleById,
};
use oxidauth_repository::{
    permissions::select_permission_by_parts::SelectPermissionByPartsQuery,
    role_permission_grants::insert_role_permission_grant::*,
    roles::select_role_by_id::SelectRoleByIdQuery,
};

pub struct CreateRolePermissionGrantUseCase<T, R, P>
where
    T: InsertRolePermissionGrantQuery,
    R: SelectRoleByIdQuery,
    P: SelectPermissionByPartsQuery,
{
    role_permission_grants: T,
    roles: R,
    permissions: P,
}

impl<T, R, P> CreateRolePermissionGrantUseCase<T, R, P>
where
    T: InsertRolePermissionGrantQuery,
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
impl<T, R, P> CreateRolePermissionGrantTrait
    for CreateRolePermissionGrantUseCase<T, R, P>
where
    T: InsertRolePermissionGrantQuery,
    R: SelectRoleByIdQuery,
    P: SelectPermissionByPartsQuery,
{
    #[tracing::instrument(
        name = "create_role_permission_grant_usecase",
        skip(self)
    )]
    async fn create_role_permission_grant(
        &self,
        params: &CreateRolePermissionGrant,
    ) -> Result<RolePermission, BoxedError> {
        self.roles
            .call(&FindRoleById {
                role_id: params.role_id,
            })
            .await?;

        let permission = self
            .permissions
            .call(&FindPermissionByParts {
                permission: params.permission.to_owned(),
            })
            .await?
            .ok_or_else(|| PermissionNotFoundError::new(&params.permission))?;

        let grant = self
            .role_permission_grants
            .call(&InsertRolePermissionGrant {
                role_id: params.role_id,
                permission_id: permission.id,
            })
            .await?;

        Ok(RolePermission { permission, grant })
    }
}
