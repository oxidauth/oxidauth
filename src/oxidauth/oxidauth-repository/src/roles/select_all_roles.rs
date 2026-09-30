pub use oxidauth_kernel::roles::Role;
use oxidauth_kernel::roles::list_all_roles::ListAllRoles;

pub use crate::prelude::*;

#[async_trait]
pub trait SelectAllRolesQuery: Send + Sync + 'static {
    async fn select_all_roles(&self, params: &ListAllRoles) -> Result<Vec<Role>, BoxedError>;
}
