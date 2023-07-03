use oxidauth_repository::role_permission_grants::insert_role_permission_grant::*;

use crate::prelude::*;

#[async_trait]
impl InsertRolePermissionGrant for Database {
    async fn insert_role_permission_grant(
        &self,
        params: &InsertRolePermissionGrantParams,
    ) -> Result<RolePermissionGrantRow, InsertRolePermissionGrantError> {
        let result = sqlx::query_as::<_, super::RolePermissionGrantRow>(include_str!(
            "./insert_role_permission_grant.sql"
        ))
        .bind(&params.id)
        .bind(&params.role_id)
        .bind(&params.permission_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertRolePermissionGrantError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_insert_a_role_permission_grant_successfully(pool: PgPool) {
        let db = Database { pool };

        let role_permission_grant_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();
        let permission_id = Uuid::new_v4();

        let insert_params = InsertRolePermissionGrantParams {
            id: Some(role_permission_grant_id),
            role_id: role_id,
            permission_id: permission_id,
        };

        match db.insert_role_permission_grant(&insert_params).await {
            Ok(role_permission_grant) => {
                assert_eq!(role_permission_grant_id, role_permission_grant.id);
                assert_eq!(role_id, role_permission_grant.role_id);
                assert_eq!(permission_id, role_permission_grant.permission_id);
            }
            Err(_) => unreachable!(),
        }
    }
}
