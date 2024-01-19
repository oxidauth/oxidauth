pub mod delete_public_key_by_id;
pub mod insert_public_key;
pub mod query_all_public_keys;
pub mod query_public_key_by_id;
pub mod select_public_key_by_user_id;

use oxidauth_kernel::public_keys::PublicKey;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgPublicKey {
    pub id: Uuid,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<PgPublicKey> for PublicKey {
    type Error = BoxedError;

    fn try_from(value: PgPublicKey) -> Result<Self, Self::Error> {
        let public_key = String::from_utf8(value.public_key)?;

        Ok(Self {
            id: value.id,
            public_key,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PgPublicSanitizedKey {
    pub id: Uuid,
    pub public_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<PgPublicSanitizedKey> for PublicKey {
    type Error = BoxedError;

    fn try_from(value: PgPublicSanitizedKey) -> Result<Self, Self::Error> {
        let public_key = String::from_utf8(value.public_key)?;

        Ok(Self {
            id: value.id,
            public_key,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}
