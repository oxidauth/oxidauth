use oxidauth_repository::permissions::query_permission_by_id::*;

use crate::prelude::*;

#[async_trait]
impl QueryPermissionById for Database {
    async fn query_permission_by_id(
        &self,
        permission_id: Uuid,
    ) -> Result<PermissionRow, QueryPermissionByIdError> {
        let result =
            sqlx::query_as::<_, super::PermissionRow>(include_str!("./query_permission_by_id.sql"))
                .bind(permission_id)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| QueryPermissionByIdError {})?;

        Ok(result)
    }
}
