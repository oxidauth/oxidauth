use oxidauth_repository::roles::query_role_by_id::*;

use crate::prelude::*;

#[async_trait]
impl QueryRoleById for Database {
    async fn query_role_by_id(&self, role_id: Uuid) -> Result<RoleRow, QueryRoleByIdError> {
        let result = sqlx::query_as::<_, super::RoleRow>(include_str!("./query_role_by_id.sql"))
            .bind(role_id)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(|_| QueryRoleByIdError {})?;

        Ok(result)
    }
}
