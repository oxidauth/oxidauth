use oxidauth_repository::roles::delete_role_by_id::*;

use crate::prelude::*;

#[async_trait]
impl DeleteRoleById for Database {
    async fn delete_role_by_id(&self, role_id: Uuid) -> Result<RoleRow, DeleteRoleByIdError> {
        let result = sqlx::query_as::<_, super::RoleRow>(include_str!("./delete_role_by_id.sql"))
            .bind(role_id)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(|_| DeleteRoleByIdError {})?;

        Ok(result)
    }
}
