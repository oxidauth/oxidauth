use oxidauth_repository::permissions::query_permission_by_parts::*;

use crate::prelude::*;

#[async_trait]
impl QueryPermissionByParts for Database {
    async fn query_permission_by_parts(
        &self,
        params: &QueryPermissionByPartsParams,
    ) -> Result<PermissionRow, QueryPermissionByPartsError> {
        let result = sqlx::query_as::<_, super::PermissionRow>(include_str!(
            "./query_permission_by_parts.sql"
        ))
        .bind(&params.realm)
        .bind(&params.resource)
        .bind(&params.action)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| QueryPermissionByPartsError {})?;

        Ok(result)
    }
}
