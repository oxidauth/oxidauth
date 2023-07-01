use oxidauth_repository::roles::insert_role::*;

use crate::prelude::*;

#[async_trait]
impl InsertRole for Database {
    async fn insert_role(&self, params: &InsertRoleParams) -> Result<RoleRow, InsertRoleError> {
        let result = sqlx::query_as::<_, super::RoleRow>(include_str!("./insert_role.sql"))
            .bind(params.id)
            .bind(params.name)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(|_| InsertRoleError {})?;

        Ok(result)
    }
}

// @GEORGE - no created at / updated at
