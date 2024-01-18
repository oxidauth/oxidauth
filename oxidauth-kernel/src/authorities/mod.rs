pub mod authenticate;
pub mod find_authority_by_client_id;
pub mod register;

pub use crate::user_authorities::UserAuthority;
pub use authenticate::*;
pub use register::*;

use crate::dev_prelude::*;

#[derive(Debug, Serialize)]
pub struct Authority {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: AuthorityStatus,
    pub strategy: AuthorityStrategy,
    pub settings: Value,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub enum AuthorityStatus {
    Enabled,
    Disabled,
}

const ENABLED: &str = "enabled";
const DISABLED: &str = "disabled";

impl TryFrom<String> for AuthorityStatus {
    type Error = BoxedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            ENABLED => Ok(Self::Enabled),
            DISABLED => Ok(Self::Disabled),
            _ => Err("invalid authority status".into()),
        }
    }
}

#[derive(Debug, Serialize)]
pub enum AuthorityStrategy {
    UsernamePassword,
    SingleUseToken,
}

const USERNAME_PASSWORD: &str = "username_password";
const SINGLE_USE_TOKEN: &str = "single_use_token";

impl TryFrom<String> for AuthorityStrategy {
    type Error = BoxedError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            USERNAME_PASSWORD => Ok(Self::UsernamePassword),
            SINGLE_USE_TOKEN => Ok(Self::SingleUseToken),
            _ => Err("invalid authority strategy".into()),
        }
    }
}
