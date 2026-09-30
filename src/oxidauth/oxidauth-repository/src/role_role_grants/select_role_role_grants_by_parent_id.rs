use oxidauth_kernel::role_role_grants::list_role_role_grants_by_parent_id::*;
pub use oxidauth_kernel::roles::Role;

pub use crate::prelude::*;

#[async_trait]
pub trait SelectRoleRoleGrantsByParentIdQuery: Send + Sync + 'static {
    async fn select_role_role_grants_by_parent_id(
        &self,
        params: &ListRoleRoleGrantsByParentId,
    ) -> Result<Vec<RoleRoleGrantDetail>, BoxedError>;
}
