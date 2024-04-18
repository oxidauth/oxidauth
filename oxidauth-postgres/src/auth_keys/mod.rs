pub mod insert_auth_key;
use oxidauth_kernel::auth_keys::create_auth_key::AuthKey;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgAuthKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: Vec<i32>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<PgAuthKey> for AuthKey {
    type Error = BoxedError;

    fn try_from(value: PgAuthKey) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            user_id: value.user_id,
            key: value.key,
            created_at: value.created_at,
        })
    }
}
