use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, service::Service, user_permission_grants::list_user_permission_grants_by_user_id::*,
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
impl<'a, T> Service<&'a ListUserPermissionGrantsByUserId>
    for ListUserPermissionGrantsByUserIdUseCase<T>
where
    T: SelectUserPermissionGrantsByUserIdQuery,
{
    type Response = Vec<UserPermission>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "list_user_permission_grants_by_user_id_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a ListUserPermissionGrantsByUserId,
    ) -> Result<Self::Response, Self::Error> {
        self.user_permission_grants
            .call(req)
            .await
    }
}
