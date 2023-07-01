use crate::prelude::*;

pub use super::RoleRow;

#[async_trait]
pub trait InsertRole: Send + Sync + 'static {
    async fn insert_role(
        &self,
        insert_role: &InsertRoleParams,
    ) -> Result<RoleRow, InsertRoleError>;
}

#[derive(Debug)]
pub struct InsertRoleParams {
    pub id: Option<Uuid>,
    pub name: String,
}

// @GEORGE - created and updated at?

#[derive(Debug)]
pub struct InsertRoleError {}
