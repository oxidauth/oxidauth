pub use oxidauth_kernel::roles::Role;
use oxidauth_kernel::roles::find_role_by_name::FindRoleByName;

pub use crate::prelude::*;

#[async_trait]
pub trait SelectRoleByNameQuery: Send + Sync + 'static {
    async fn select_role_by_name(&self, params: &FindRoleByName) -> Result<Role, BoxedError>;
}
