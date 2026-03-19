use async_trait::async_trait;

use oxidauth_kernel::{
    error::BoxedError,
    role_role_grants::list_role_role_grants_by_parent_id::{
        ListRoleRoleGrantsByParentId, ListRoleRoleGrantsByParentIdTrait,
        RoleRoleGrantDetail,
    },
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
impl<T> ListRoleRoleGrantsByParentIdTrait
    for ListRoleRoleGrantsByParentIdUseCase<T>
where
    T: SelectRoleRoleGrantsByParentIdQuery,
{
    #[tracing::instrument(
        name = "list_role_role_grants_by_parent_id_usecase",
        skip(self)
    )]
    async fn list_role_role_grants_by_parent_id(
        &self,
        params: &ListRoleRoleGrantsByParentId,
    ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError> {
        self.role_role_grants
            .call(params)
            .await
    }
}
