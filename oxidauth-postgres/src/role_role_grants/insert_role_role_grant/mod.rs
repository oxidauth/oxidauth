use oxidauth_repository::role_role_grants::insert_role_role_grant::*;

use crate::prelude::*;

#[async_trait]
impl InsertRoleRoleGrant for Database {
    async fn insert_role_role_grant(
        &self,
        params: &InsertRoleRoleGrantParams,
    ) -> Result<RoleRoleGrantRow, InsertRoleRoleGrantError> {
        let result = sqlx::query_as::<_, super::RoleRoleGrantRow>(include_str!(
            "./insert_role_role_grant.sql"
        ))
        .bind(&params.parent_id)
        .bind(&params.child_id)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|_| InsertRoleRoleGrantError {})?;

        Ok(result)
    }
}
// @GEORGE - also leaving this one because I'm not sure if table will have id column or not
// #[cfg(test)]
// mod tests {
//     use sqlx::PgPool;

//     use super::*;

//     #[sqlx::test]
//     async fn it_should_insert_an_authority_successfully(pool: PgPool) {
//         let db = Database { pool };

//         let authority_id = Uuid::new_v4();

//         let insert_params = InsertAuthorityParams {
//             id: Some(authority_id),
//             name: "Test".to_string(),
//             client_key: Uuid::new_v4(),
//             status: "Test".to_string(),
//             strategy: "Test".to_string(),
//             settings: serde_json::Value::default(),
//             params: serde_json::Value::default(),
//         };

//         match db.insert_authority(&insert_params).await {
//             Ok(authority) => {
//                 assert_eq!(authority_id, authority.id);
//                 assert_eq!(insert_params.name, authority.name);
//                 assert_eq!(insert_params.client_key, authority.client_key);
//                 assert_eq!(insert_params.status, authority.status);
//                 assert_eq!(insert_params.strategy, authority.strategy);
//                 assert_eq!(insert_params.settings, authority.settings);
//                 assert_eq!(insert_params.params, authority.params);
//             }
//             Err(_) => unreachable!(),
//         }
//     }
// }
