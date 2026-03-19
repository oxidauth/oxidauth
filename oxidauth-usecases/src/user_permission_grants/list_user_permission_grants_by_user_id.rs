use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    user_permission_grants::list_user_permission_grants_by_user_id::{
        ListUserPermissionGrantsByUserId,
        ListUserPermissionGrantsByUserIdTrait, UserPermission,
    },
};
use oxidauth_repository::user_permission_grants::select_user_permission_grants_by_user_id::SelectUserPermissionGrantsByUserIdQuery;

pub struct ListUserPermissionGrantsByUserIdUseCase<T>
where
    T: SelectUserPermissionGrantsByUserIdQuery,
{
    user_permission_grants: T,
}

impl<T> ListUserPermissionGrantsByUserIdUseCase<T>
where
    T: SelectUserPermissionGrantsByUserIdQuery,
{
    pub fn new(user_permission_grants: T) -> Self {
        Self {
            user_permission_grants,
        }
    }
}

#[async_trait]
impl<T> ListUserPermissionGrantsByUserIdTrait
    for ListUserPermissionGrantsByUserIdUseCase<T>
where
    T: SelectUserPermissionGrantsByUserIdQuery,
{
    #[tracing::instrument(
        name = "list_user_permission_grants_by_user_id_usecase",
        skip(self)
    )]
    async fn list_user_permission_grants_by_user_id(
        &self,
        params: &ListUserPermissionGrantsByUserId,
    ) -> Result<Vec<UserPermission>, BoxedError> {
        self.user_permission_grants
            .call(params)
            .await
    }
}
