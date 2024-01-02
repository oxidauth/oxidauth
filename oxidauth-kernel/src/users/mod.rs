pub mod find_user_by_id;
pub mod user_create;

use core::fmt;
use std::str::FromStr;

use crate::dev_prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub kind: UserKind,
    pub status: UserStatus,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserKind {
    Human,
    Api,
}

pub const HUMAN: &str = "human";
pub const API: &str = "api";

impl From<&UserKind> for &'static str {
    fn from(user_kind: &UserKind) -> Self {
        match user_kind {
            UserKind::Human => HUMAN,
            UserKind::Api => API,
        }
    }
}

impl FromStr for UserKind {
    type Err = ParseUserKindErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_kind = match s {
            HUMAN => UserKind::Human,
            API => UserKind::Api,
            _ => {
                return Err(ParseUserKindErr {
                    unknown: s.to_owned(),
                })
            },
        };

        Ok(user_kind)
    }
}

#[derive(Debug)]
pub struct ParseUserKindErr {
    unknown: String,
}

impl fmt::Display for ParseUserKindErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to parse user_kind, unknown: {}",
            self.unknown
        )
    }
}

impl std::error::Error for ParseUserKindErr {}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserStatus {
    Enabled,
    Disabled,
}

pub const ENABLED: &str = "enabled";
pub const DISABLED: &str = "disabled";

impl From<&UserStatus> for &'static str {
    fn from(status: &UserStatus) -> Self {
        match status {
            UserStatus::Enabled => ENABLED,
            UserStatus::Disabled => DISABLED,
        }
    }
}

impl FromStr for UserStatus {
    type Err = ParseUserStatusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_status = match s {
            "enabled" => UserStatus::Enabled,
            "disabled" => UserStatus::Disabled,
            _ => {
                return Err(ParseUserStatusErr {
                    unknown: s.to_owned(),
                })
            },
        };

        Ok(user_status)
    }
}

#[derive(Debug)]
pub struct ParseUserStatusErr {
    unknown: String,
}

impl fmt::Display for ParseUserStatusErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to parse user_kind, unknown: {}",
            self.unknown
        )
    }
}

impl std::error::Error for ParseUserStatusErr {}
