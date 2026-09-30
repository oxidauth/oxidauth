use oxidauth_kernel::role_role_grants::delete_role_role_grant::{DeleteRoleRoleGrant, *};
pub use oxidauth_kernel::roles::Role;

pub use crate::prelude::*;

#[async_trait]
pub trait DeleteRoleRoleGrantQuery: Send + Sync + 'static {
    async fn delete_role_role_grant(
        &self,
        params: &DeleteRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError>;
}
