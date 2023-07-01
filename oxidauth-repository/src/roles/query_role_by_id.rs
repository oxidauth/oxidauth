use crate::prelude::*;

pub use super::RoleRow;

#[async_trait]
pub trait QueryRoleById: Send + Sync + 'static {
    async fn query_role_by_id(
        &self,
        role_id: Uuid,
    ) -> Result<RoleRow, QueryRoleByIdError>;
}

#[derive(Debug)]
pub struct QueryRoleByIdError {}
