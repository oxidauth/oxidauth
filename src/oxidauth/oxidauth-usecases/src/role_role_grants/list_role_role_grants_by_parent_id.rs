use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    role_role_grants::list_role_role_grants_by_parent_id::*,
};
use oxidauth_repository::role_role_grants::select_role_role_grants_by_parent_id::SelectRoleRoleGrantsByParentIdQuery;

pub struct ListRoleRoleGrantsByParentIdUseCase<T>
where
    T: SelectRoleRoleGrantsByParentIdQuery,
{
    role_role_grants: T,
}

impl<T> ListRoleRoleGrantsByParentIdUseCase<T>
where
    T: SelectRoleRoleGrantsByParentIdQuery,
{
    pub fn new(role_role_grants: T) -> Self {
        Self { role_role_grants }
    }
}

#[async_trait]
impl<'a, T> Service<&'a ListRoleRoleGrantsByParentId>
    for ListRoleRoleGrantsByParentIdUseCase<T>
where
    T: SelectRoleRoleGrantsByParentIdQuery,
{
    type Response = Vec<RoleRoleGrantDetail>;
    type Error = BoxedError;

    #[tracing::instrument(
        name = "list_role_role_grants_by_parent_id_usecase",
        skip(self)
    )]
    async fn call(
        &self,
        req: &'a ListRoleRoleGrantsByParentId,
    ) -> Result<Self::Response, Self::Error> {
        self.role_role_grants
            .call(req)
            .await
    }
}
