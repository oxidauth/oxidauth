pub use oxidauth_kernel::roles::Role;
use oxidauth_kernel::roles::create_role::CreateRole;

pub use crate::prelude::*;

#[async_trait]
pub trait InsertRoleQuery: Send + Sync + 'static {
    async fn insert_role(&self, params: &CreateRole) -> Result<Role, BoxedError>;
}
