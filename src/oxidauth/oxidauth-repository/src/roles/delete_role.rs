pub use oxidauth_kernel::roles::Role;
use oxidauth_kernel::roles::delete_role::DeleteRole;

pub use crate::prelude::*;

#[async_trait]
pub trait DeleteRoleQuery: Send + Sync + 'static {
    async fn delete_role(&self, params: &DeleteRole) -> Result<Role, BoxedError>;
}
