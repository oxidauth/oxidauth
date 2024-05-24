pub mod insert_totp_secret;
pub mod select_totp_secret_by_user_id;

use oxidauth_kernel::totp_secrets::TOTPSecret;

#[derive(Debug, sqlx::FromRow)]
pub struct PgTotpSecret {
    pub totp_secret: String,
}

impl From<PgTotpSecret> for TOTPSecret {
    fn from(value: PgTotpSecret) -> Self {
        Self {
            secret: value.totp_secret,
        }
    }
}
