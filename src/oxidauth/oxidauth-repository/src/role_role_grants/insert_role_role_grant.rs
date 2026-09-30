use oxidauth_kernel::role_role_grants::create_role_role_grant::*;
pub use oxidauth_kernel::roles::Role;

pub use crate::prelude::*;

#[async_trait]
pub trait InsertRoleRoleGrantQuery: Send + Sync + 'static {
    async fn insert_role_role_grant(
        &self,
        params: &CreateRoleRoleGrant,
    ) -> Result<RoleRoleGrant, BoxedError>;
}
