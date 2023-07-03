use oxidauth_repository::user_role_grants::insert_user_role_grant::*;

use crate::prelude::*;

#[async_trait]
impl InsertUserRoleGrant for Database {
    async fn insert_user_role_grant(
        &self,
        params: &InsertUserRoleGrantParams,
    ) -> Result<UserRoleGrantRow, InsertUserRoleGrantError> {
        let result = sqlx::query_as::<_, super::UserRoleGrantRow>(include_str!(
            "./insert_user_role_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.role_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertUserRoleGrantError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_insert_a_user_role_grant_successfully(pool: PgPool) {
        let db = Database { pool };

        let user_id = Uuid::new_v4();
        let role_id = Uuid::new_v4();

        let insert_params = InsertUserRoleGrantParams {
            user_id: user_id,
            role_id: role_id,
        };

        match db.insert_user_role_grant(&insert_params).await {
            Ok(user_role_grant) => {
                assert_eq!(user_id, user_role_grant.user_id);
                assert_eq!(role_id, user_role_grant.role_id);
            }
            Err(_) => unreachable!(),
        }
    }
}
