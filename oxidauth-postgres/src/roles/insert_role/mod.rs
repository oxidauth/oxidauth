use oxidauth_repository::roles::insert_role::*;

use crate::prelude::*;

#[async_trait]
impl InsertRole for Database {
    async fn insert_role(&self, params: &InsertRoleParams) -> Result<RoleRow, InsertRoleError> {
        let result = sqlx::query_as::<_, super::RoleRow>(include_str!("./insert_role.sql"))
            .bind(&params.id)
            .bind(&params.name)
            .fetch_one(&self.pool)
            .await
            .map(Into::into)
            .map_err(|_| InsertRoleError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_insert_a_role_successfully(pool: PgPool) {
        let db = Database { pool };

        let role_id = Uuid::new_v4();

        let insert_params = InsertRoleParams {
            id: Some(role_id),
            name: "Test".to_string(),
        };

        match db.insert_role(&insert_params).await {
            Ok(role) => {
                assert_eq!(role_id, role.id);
                assert_eq!(insert_params.name, role.name);
            }
            Err(_) => unreachable!(),
        }
    }
}
