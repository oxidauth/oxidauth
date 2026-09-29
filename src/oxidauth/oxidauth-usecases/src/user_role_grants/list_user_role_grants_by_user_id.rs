use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError, service::Service,
    user_role_grants::list_user_role_grants_by_user_id::*,
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
impl<'a, T> Service<&'a ListUserRoleGrantsByUserId>
    for ListUserRoleGrantsByUserIdUseCase<T>
where
    T: SelectUserRoleGrantsByUserIdQuery,
{
    type Response = Vec<UserRole>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "list_user_role_grants_by_user_id_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a ListUserRoleGrantsByUserId,
    ) -> Result<Self::Response, Self::Error> {
        self.user_role_grants
            .call(req)
            .await
    }
}
