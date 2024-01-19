use oxidauth_repository::authorities::update_authority::*;

use crate::prelude::*;

#[async_trait]
impl UpdateAuthority for Database {
    async fn update_authority(
        &self,
        params: &UpdateAuthorityParams,
    ) -> Result<AuthorityRow, UpdateAuthorityError> {
        let result =
            sqlx::query_as::<_, super::PgAuthority>(include_str!("./update_authority.sql"))
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
                .map_err(|_| UpdateAuthorityError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::authorities::insert_authority::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_update_an_authority_successfully(pool: PgPool) {
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

        let update_params = UpdateAuthorityParams {
            id: authority_id,
            name: "Updated Name".to_string(),
            client_key: Uuid::new_v4(),
            status: "Updated Status".to_string(),
            strategy: "Updated Strategy".to_string(),
            settings: serde_json::Value::default(),
            params: serde_json::Value::default(),
        };

        // @GEORGE - want eyes here too
        db.insert_authority(&insert_params)
            .await
            .expect("should be able to insert authority");

        match db.update_authority(&update_params).await {
            Ok(authority) => {
                assert_eq!(authority_id, authority.id);
                assert_eq!(update_params.name, authority.name);
                assert_eq!(update_params.status, authority.status);
                assert_eq!(update_params.strategy, authority.strategy);
            }
            Err(_) => unreachable!(),
        }
    }
}
