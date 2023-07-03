use oxidauth_repository::authorities::insert_authority::*;

use crate::prelude::*;

#[async_trait]
impl InsertAuthority for Database {
    async fn insert_authority(
        &self,
        params: &InsertAuthorityParams,
    ) -> Result<AuthorityRow, InsertAuthorityError> {
        let result =
            sqlx::query_as::<_, super::AuthorityRow>(include_str!("./insert_authority.sql"))
                .bind(&params.id)
                .bind(&params.name)
                .bind(&params.client_key)
                .bind(&params.status)
                .bind(&params.strategy)
                .bind(&params.settings)
                .bind(&params.params)
                .fetch_one(&self.pool)
                .await
                .map(Into::into)
                .map_err(|_| InsertAuthorityError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_insert_an_authority_successfully(pool: PgPool) {
        let db = Database { pool };

        let authority_id = Uuid::new_v4();

        let insert_params = InsertAuthorityParams {
            id: Some(authority_id),
            name: "Test".to_string(),
            client_key: Uuid::new_v4(),
            status: "Test".to_string(),
            strategy: "Test".to_string(),
            settings: serde_json::Value::default(),
            params: serde_json::Value::default(),
        };

        match db.insert_authority(&insert_params).await {
            Ok(authority) => {
                assert_eq!(authority_id, authority.id);
                assert_eq!(insert_params.name, authority.name);
                assert_eq!(insert_params.client_key, authority.client_key);
                assert_eq!(insert_params.status, authority.status);
                assert_eq!(insert_params.strategy, authority.strategy);
                assert_eq!(insert_params.settings, authority.settings);
                assert_eq!(insert_params.params, authority.params);
            }
            Err(_) => unreachable!(),
        }
    }
}
