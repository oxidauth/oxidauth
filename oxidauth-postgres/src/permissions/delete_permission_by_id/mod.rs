use oxidauth_repository::permissions::delete_permission_by_id::*;

use crate::prelude::*;

#[async_trait]
impl DeletePermissionById for Database {
    async fn delete_permission_by_id(
        &self,
        permission_id: Uuid,
    ) -> Result<PermissionRow, DeletePermissionByIdError> {
        let result = sqlx::query_as::<_, super::PermissionRow>(include_str!(
            "./delete_permission_by_id.sql"
        ))
        .bind(permission_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| DeletePermissionByIdError {})?;

        Ok(result)
    }
}
