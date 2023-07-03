use oxidauth_repository::roles::update_role::*;

use crate::prelude::*;

#[async_trait]
impl UpdateRole for Database {
    async fn update_role(&self, params: &UpdateRoleParams) -> Result<RoleRow, UpdateRoleError> {
        let result = sqlx::query_as::<_, super::RoleRow>(include_str!("./update_role.sql"))
            .bind(&params.id)
            .bind(&params.name)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(|_| UpdateRoleError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::roles::insert_role::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_update_a_role_successfully(pool: PgPool) {
        let db = Database { pool };

        let role_id = Uuid::new_v4();

        let insert_params = InsertRoleParams {
            id: Some(role_id),
            name: "Test".to_string(),
        };

        let update_params = UpdateRoleParams {
            id: role_id,
            name: "UpdateName".to_string(),
        };

        db.insert_role(&insert_params)
            .await
            .expect("should be able to insert role");

        match db.update_role(&update_params).await {
            Ok(role) => {
                assert_eq!(role_id, role.id);
                assert_eq!(insert_params.name, role.name);
            }
            Err(_) => unreachable!(),
        }
    }
}
