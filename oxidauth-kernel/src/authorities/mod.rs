use std::{error::Error, fmt, str::FromStr};

use serde::Serialize;

pub mod authenticate;
pub mod create_authority;
pub mod delete_authority;
pub mod find_authority_by_client_id;
pub mod find_authority_by_id;
pub mod find_authority_by_strategy;
pub mod list_all_authorities;
pub mod register;
pub mod update_authority;

pub use crate::user_authorities::UserAuthority;
pub use authenticate::*;
pub use register::*;

use crate::dev_prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Authority {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: AuthorityStatus,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthoritySettings {
    pub jwt_ttl: std::time::Duration,
    pub refresh_token_ttl: std::time::Duration,
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
}

impl AuthorityStrategy {}

const USERNAME_PASSWORD: &str = "username_password";
const SINGLE_USE_TOKEN: &str = "single_use_token";

impl fmt::Display for AuthorityStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AuthorityStrategy::*;

        match self {
            UsernamePassword => write!(f, "{}", USERNAME_PASSWORD),
            SingleUseToken => write!(f, "{}", SINGLE_USE_TOKEN),
        }
    }
}

impl FromStr for AuthorityStrategy {
    type Err = ParseAuthorityStrategyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            USERNAME_PASSWORD => AuthorityStrategy::UsernamePassword,
            SINGLE_USE_TOKEN => AuthorityStrategy::SingleUseToken,
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
}

impl AuthorityNotFoundError {
    pub fn strategy(strategy: AuthorityStrategy) -> Box<Self> {
        Box::new(Self::Strategy(strategy))
    }

    pub fn id(id: Uuid) -> Box<Self> {
        Box::new(Self::Id(id))
    }
}

impl fmt::Display for AuthorityNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let missing = match self {
            AuthorityNotFoundError::Strategy(strategy) => strategy.to_string(),
            AuthorityNotFoundError::Id(id) => id.to_string(),
        };

        write!(
            f,
            "authority not found: {}",
            missing
        )
    }
}

impl Error for AuthorityNotFoundError {}
