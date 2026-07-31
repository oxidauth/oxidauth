use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    role_role_grants::list_role_role_grants_by_child_id::*,
};
use oxidauth_repository::role_role_grants::select_role_role_grants_by_child_id::SelectRoleRoleGrantsByChildIdQuery;

pub struct ListRoleRoleGrantsByChildIdUseCase<T>
where
    T: SelectRoleRoleGrantsByChildIdQuery,
{
    role_role_grants: T,
}

impl<T> ListRoleRoleGrantsByChildIdUseCase<T>
where
    T: SelectRoleRoleGrantsByChildIdQuery,
{
    pub fn new(role_role_grants: T) -> Self {
        Self { role_role_grants }
    }
}

#[async_trait]
impl<'a, T> Service<&'a ListRoleRoleGrantsByChildId>
    for ListRoleRoleGrantsByChildIdUseCase<T>
where
    T: SelectRoleRoleGrantsByChildIdQuery,
{
    type Response = Vec<RoleRoleGrantDetail>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "list_role_role_grants_by_child_id_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a ListRoleRoleGrantsByChildId,
    ) -> Result<Self::Response, Self::Error> {
        self.role_role_grants
            .call(req)
            .await
    }
}
