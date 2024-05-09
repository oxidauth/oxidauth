pub mod insert_totp_secret;
use oxidauth_kernel::totp_secrets::create_totp_secret::TotpSecret;

use crate::prelude::*;

#[derive(Debug, sqlx::FromRow)]
pub struct PgTotpSecret {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key: Vec<i32>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<PgTotpSecret> for TotpSecret {
    type Error = BoxedError;

    fn try_from(value: PgTotpSecret) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            user_id: value.user_id,
            key: value.key,
            created_at: value.created_at,
        })
    }
}
