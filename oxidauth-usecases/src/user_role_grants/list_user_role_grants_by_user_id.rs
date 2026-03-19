use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    user_role_grants::list_user_role_grants_by_user_id::{
        ListUserRoleGrantsByUserId, ListUserRoleGrantsByUserIdTrait,
        UserRole,
    },
};
use oxidauth_repository::user_role_grants::select_user_role_grants_by_user_id::SelectUserRoleGrantsByUserIdQuery;

pub struct ListUserRoleGrantsByUserIdUseCase<T>
where
    T: SelectUserRoleGrantsByUserIdQuery,
{
    user_role_grants: T,
}

impl<T> ListUserRoleGrantsByUserIdUseCase<T>
where
    T: SelectUserRoleGrantsByUserIdQuery,
{
    pub fn new(user_role_grants: T) -> Self {
        Self { user_role_grants }
    }
}

#[async_trait]
impl<T> ListUserRoleGrantsByUserIdTrait
    for ListUserRoleGrantsByUserIdUseCase<T>
where
    T: SelectUserRoleGrantsByUserIdQuery,
{
    #[tracing::instrument(
        name = "list_user_role_grants_by_user_id_usecase",
        skip(self)
    )]
    async fn list_user_role_grants_by_user_id(
        &self,
        params: &ListUserRoleGrantsByUserId,
    ) -> Result<Vec<UserRole>, BoxedError> {
        self.user_role_grants
            .call(params)
            .await
    }
}
