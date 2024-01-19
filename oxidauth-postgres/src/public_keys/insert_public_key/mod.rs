use oxidauth_kernel::public_keys::PublicKey;
use oxidauth_repository::public_keys::insert_public_key::*;

use crate::prelude::*;

use super::PgPublicKey;

#[async_trait]
impl InsertPublicKey for Database {
    async fn insert_public_key(
        &self,
        params: &InsertPublicKeyParams,
    ) -> Result<PublicKey, InsertPublicKeyError> {
        let result = sqlx::query_as::<_, PgPublicKey>(include_str!(
            "./insert_public_key.sql"
        ))
        .bind(params.id)
        .bind(&params.private_key)
        .bind(&params.public_key)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| InsertPublicKeyError {})?
        .try_into()
        .map_err(|_| InsertPublicKeyError {})?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use oxidauth_repository::public_keys::insert_public_key::*;
    use sqlx::PgPool;

    use super::*;

    #[sqlx::test]
    async fn it_should_a_public_key_successfully(pool: PgPool) {
        let db = Database { pool };

        let public_key_id = Uuid::new_v4();

        // @GEORGE - again just guessing
        let insert_params = InsertPublicKeyParams {
            id: Some(public_key_id),
            public_key: [1, 1, 1, 1].to_vec(),
            private_key: [2, 2, 2, 2].to_vec(),
        };

        match db
            .insert_public_key(&insert_params)
            .await
        {
            Ok(public_key) => {
                let insert_public_key =
                    String::from_utf8(insert_params.public_key)
                        .expect("can parse from utf8");

                assert_eq!(public_key_id, public_key.id);
                assert_eq!(
                    insert_public_key,
                    public_key.public_key
                );
            },
            Err(_) => unreachable!(),
        }
    }
}
