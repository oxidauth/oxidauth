pub use oxidauth_kernel::roles::Role;
use oxidauth_kernel::roles::find_role_by_id::FindRoleById;

pub use crate::prelude::*;

#[async_trait]
pub trait SelectRoleByIdQuery: Send + Sync + 'static {
    async fn select_role_by_id(&self, params: &FindRoleById) -> Result<Role, BoxedError>;
}
