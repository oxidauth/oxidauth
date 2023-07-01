use oxidauth_repository::roles::update_role::*;

use crate::prelude::*;

#[async_trait]
impl UpdateRole for Database {
    async fn update_role(&self, params: &UpdateRoleParams) -> Result<RoleRow, UpdateRoleError> {
        let result = sqlx::query_as::<_, super::RoleRow>(include_str!("./update_role.sql"))
            .bind(params.id)
            .bind(params.name)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(|_| UpdateRoleError {})?;

        Ok(result)
    }
}
