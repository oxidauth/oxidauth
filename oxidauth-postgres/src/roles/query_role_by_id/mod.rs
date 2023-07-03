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

#[cfg(test)]
mod tests {
    use oxidauth_repository::roles::insert_role::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_query_a_role_by_id_successfully(pool: PgPool) {
        let db = Database { pool };

        let role_id = Uuid::new_v4();

        let insert_params = InsertRoleParams {
            id: Some(role_id),
            name: "Test".to_string(),
        };

        db.insert_role(&insert_params)
            .await
            .expect("should be able to insert role");

        match db.query_role_by_id(role_id).await {
            Ok(role) => {
                assert_eq!(role_id, role.id);
                assert_eq!(insert_params.name, role.name);
            }
            Err(_) => unreachable!(),
        }
    }
}
