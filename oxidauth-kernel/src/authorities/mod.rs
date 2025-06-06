use std::{error::Error, fmt, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};
use url::Url;

pub mod create_authority;
pub mod delete_authority;
pub mod find_authority_by_client_key;
pub mod find_authority_by_id;
pub mod find_authority_by_strategy;
pub mod list_all_authorities;
pub mod update_authority;

pub use crate::user_authorities::UserAuthority;

use crate::{JsonValue, dev_prelude::*, jwt::EntitlementsEncoding};

#[derive(Debug, Serialize, Deserialize)]
pub struct Authority {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: AuthorityStatus,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthoritySettings {
    pub jwt_ttl: Duration,
    pub jwt_nbf_offset: NbfOffset,
    pub refresh_token_ttl: Duration,
    pub totp: TotpSettings,
    pub entitlements_encoding: EntitlementsEncoding,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NbfOffset {
    Enabled(Duration),
    Disabled,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TotpSettings {
    Enabled {
        totp_ttl: Duration,
        webhook: Url,
        webhook_key: String,
    },
    Disabled,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStatus {
    Enabled,
    Disabled,
}

const ENABLED: &str = "enabled";
const DISABLED: &str = "disabled";

impl fmt::Display for AuthorityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AuthorityStatus::*;

        match self {
            Enabled => write!(f, "{}", ENABLED),
            Disabled => write!(f, "{}", DISABLED),
        }
    }
}

impl FromStr for AuthorityStatus {
    type Err = BoxedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ENABLED => Ok(AuthorityStatus::Enabled),
            DISABLED => Ok(AuthorityStatus::Disabled),
            status => Err(format!(
                "invalid authority status: {}",
                status
            )
            .into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStrategy {
    UsernamePassword,
    SingleUseToken,
    Oauth2,
}

impl AuthorityStrategy {}

const USERNAME_PASSWORD: &str = "username_password";
const SINGLE_USE_TOKEN: &str = "single_use_token";
const OAUTH2: &str = "oauth2";

impl fmt::Display for AuthorityStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AuthorityStrategy::*;

        match self {
            UsernamePassword => write!(f, "{}", USERNAME_PASSWORD),
            SingleUseToken => write!(f, "{}", SINGLE_USE_TOKEN),
            Oauth2 => write!(f, "{}", OAUTH2),
        }
    }
}

impl FromStr for AuthorityStrategy {
    type Err = ParseAuthorityStrategyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            USERNAME_PASSWORD => AuthorityStrategy::UsernamePassword,
            SINGLE_USE_TOKEN => AuthorityStrategy::SingleUseToken,
            OAUTH2 => AuthorityStrategy::Oauth2,
            strategy => {
                return Err(
                    ParseAuthorityStrategyError::Unknown(strategy.to_owned()),
                );
            },
        };

        Ok(res)
    }
}

#[derive(Debug)]
pub enum ParseAuthorityStrategyError {
    Unknown(String),
}

impl fmt::Display for ParseAuthorityStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseAuthorityStrategyError::*;

        match self {
            Unknown(value) => write!(
                f,
                "unknown authority strategy: {}",
                value
            ),
        }
    }
}

impl Error for ParseAuthorityStrategyError {}

#[derive(Debug)]
pub enum AuthorityNotFoundError {
    Strategy(AuthorityStrategy),
    Id(Uuid),
    ClientKey(Uuid),
}

impl AuthorityNotFoundError {
    pub fn strategy(strategy: AuthorityStrategy) -> Box<Self> {
        Box::new(Self::Strategy(strategy))
    }

    pub fn id(id: Uuid) -> Box<Self> {
        Box::new(Self::Id(id))
    }

    pub fn client_key(id: Uuid) -> Box<Self> {
        Box::new(Self::ClientKey(id))
    }
}

impl fmt::Display for AuthorityNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthorityNotFoundError::Strategy(strategy) => write!(
                f,
                "authority not found by strategy: {}",
                strategy
            ),
            AuthorityNotFoundError::Id(id) => write!(
                f,
                "authority not found by id: {}",
                id,
            ),
            AuthorityNotFoundError::ClientKey(id) => write!(
                f,
                "authority not found by client_key: {}",
                id,
            ),
        }
    }
}

impl Error for AuthorityNotFoundError {}
