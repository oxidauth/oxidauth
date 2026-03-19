use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    roles::find_role_by_id::FindRoleById,
    user_role_grants::create_user_role_grant::{
        CreateUserRoleGrant, CreateUserRoleGrantTrait, UserRole,
    },
    users::find_user_by_id::FindUserById,
};
use oxidauth_repository::{
    roles::select_role_by_id::SelectRoleByIdQuery,
    user_role_grants::insert_user_role_grant::InsertUserRoleGrantQuery,
    users::select_user_by_id_query::SelectUserByIdQuery,
};

pub struct CreateUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: InsertUserRoleGrantQuery,
{
    users: U,
    roles: R,
    user_role_grants: UR,
}

impl<U, R, UR> CreateUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: InsertUserRoleGrantQuery,
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
impl<U, R, UR> CreateUserRoleGrantTrait
    for CreateUserRoleGrantUseCase<U, R, UR>
where
    U: SelectUserByIdQuery,
    R: SelectRoleByIdQuery,
    UR: InsertUserRoleGrantQuery,
{
    #[tracing::instrument(name = "create_user_role_grant_usecase", skip(self))]
    async fn create_user_role_grant(
        &self,
        params: &CreateUserRoleGrant,
    ) -> Result<UserRole, BoxedError> {
        let user = self
            .users
            .call(&FindUserById {
                user_id: params.user_id,
            })
            .await?;

        let role = self
            .roles
            .call(&FindRoleById {
                role_id: params.role_id,
            })
            .await?;

        let grant = self
            .user_role_grants
            .call(&CreateUserRoleGrant {
                user_id: user.id,
                role_id: role.id,
            })
            .await?;

        Ok(UserRole { role, grant })
    }
}
