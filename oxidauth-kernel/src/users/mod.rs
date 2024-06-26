pub mod create_user;
pub mod delete_user_by_id;
pub mod find_user_by_id;
pub mod find_user_by_username;
pub mod find_users_by_ids;
pub mod list_all_users;
pub mod update_user;

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

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UserKind {
    #[default]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    #[default]
    Enabled,
    Invited,
    Disabled,
}

pub const ENABLED: &str = "enabled";
pub const INVITED: &str = "invited";
pub const DISABLED: &str = "disabled";

impl From<&UserStatus> for &'static str {
    fn from(status: &UserStatus) -> Self {
        match status {
            UserStatus::Enabled => ENABLED,
            UserStatus::Invited => INVITED,
            UserStatus::Disabled => DISABLED,
        }
    }
}

impl FromStr for UserStatus {
    type Err = ParseUserStatusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_status = match s {
            "enabled" => UserStatus::Enabled,
            "invited" => UserStatus::Invited,
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

#[derive(Debug, Clone, Deserialize)]
pub struct Username(pub String);

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Username {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
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

#[derive(Debug)]
pub enum UserNotFoundError {
    Username(Username),
    Id(Uuid),
}

impl UserNotFoundError {
    pub fn username(username: &Username) -> Box<Self> {
        Box::new(Self::Username(
            username.clone(),
        ))
    }

    pub fn id(id: Uuid) -> Box<Self> {
        Box::new(Self::Id(id))
    }
}

impl fmt::Display for UserNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let missing = match self {
            UserNotFoundError::Username(username) => {
                format!("username == {}", username)
            },
            UserNotFoundError::Id(id) => format!("id == {}", id),
        };

        write!(
            f,
            "user not found where: {}",
            missing
        )
    }
}

impl std::error::Error for UserNotFoundError {}
