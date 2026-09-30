pub use oxidauth_kernel::roles::Role;
use oxidauth_kernel::roles::update_role::UpdateRole;

pub use crate::prelude::*;

#[async_trait]
pub trait UpdateRoleQuery: Send + Sync + 'static {
    async fn update_role(&self, params: &UpdateRole) -> Result<Role, BoxedError>;
}
