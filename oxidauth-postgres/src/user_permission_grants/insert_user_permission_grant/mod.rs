use oxidauth_repository::user_permission_grants::insert_user_permission_grant::*;

use crate::prelude::*;

#[async_trait]
impl InsertUserPermissionGrant for Database {
    async fn insert_user_permission_grant(
        &self,
        params: &InsertUserPermissionGrantParams,
    ) -> Result<UserPermissionGrantRow, InsertUserPermissionGrantError> {
        let result = sqlx::query_as::<_, super::UserPermissionGrantRow>(include_str!(
            "./insert_user_permission_grant.sql"
        ))
        .bind(params.user_id)
        .bind(params.permission_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertUserPermissionGrantError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[ignore]
    #[sqlx::test]
    async fn it_should_insert_a_user_permission_grant_successfully(pool: PgPool) {
        // let db = Database { pool };
        //
        // let user_id = Uuid::new_v4();
        // let permission_id = Uuid::new_v4();
        //
        // let insert_params = InsertUserRoleGrantParams {
        //     user_id: user_id,
        //     permission_id: permission_id,
        // };
        //
        // match db.insert_user_permission_grant(&insert_params).await {
        //     Ok(user_permission_grant) => {
        //         assert_eq!(user_id, user_permission_grant.user_id);
        //         assert_eq!(permission_id, user_permission_grant.permission_id);
        //     }
        //     Err(_) => unreachable!(),
        // }
    }
}
