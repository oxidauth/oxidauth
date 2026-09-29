use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    roles::find_role_by_id::FindRoleById,
    service::Service,
    user_role_grants::{delete_user_role_grant::*, UserRole},
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    roles::select_role_by_id::SelectRoleByIdQuery,
    user_role_grants::delete_user_role_grant::DeleteUserRoleGrantQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

pub struct DeleteUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: DeleteUserRoleGrantQuery,
{
    users: U,
    roles: R,
    user_role_grants: UR,
}

impl<U, R, UR> DeleteUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: DeleteUserRoleGrantQuery,
{
    pub fn new(users: U, roles: R, user_role_grants: UR) -> Self {
        Self {
            users,
            roles,
            user_role_grants,
        }
    }
}

#[async_trait]
impl<'a, U, R, UR> Service<&'a DeleteUserRoleGrant>
    for DeleteUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: DeleteUserRoleGrantQuery,
{
    type Response = UserRole;
    type Error = BoxedError;

    #[tracing::instrument(name = "delete_user_role_grant_usecase", skip(self))]
    async fn call(
        &self,
        req: &'a DeleteUserRoleGrant,
    ) -> Result<Self::Response, Self::Error> {
        let user = self
            .users
            .call(&FindUserById {
                user_id: req.user_id,
            })
            .await?;

        let role = self
            .roles
            .call(&FindRoleById {
                role_id: req.role_id,
            })
            .await?;

        let grant = self
            .user_role_grants
            .call(&DeleteUserRoleGrant {
                user_id: user.id,
                role_id: role.id,
            })
            .await?;

        Ok(UserRole { role, grant })
    }
}
