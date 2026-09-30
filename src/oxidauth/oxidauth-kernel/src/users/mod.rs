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
                });
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
        write!(f, "failed to parse user_kind, unknown: {}", self.unknown)
    }
}

impl std::error::Error for ParseUserKindErr {
}

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
            ENABLED => UserStatus::Enabled,
            INVITED => UserStatus::Invited,
            DISABLED => UserStatus::Disabled,
            _ => {
                return Err(ParseUserStatusErr {
                    unknown: s.to_owned(),
                });
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
        write!(f, "failed to parse user_status, unknown: {}", self.unknown)
    }
}

impl std::error::Error for ParseUserStatusErr {
}

#[derive(Debug)]
pub enum UserNotFoundError {
    Username(Username),
    Id(Uuid),
}

impl UserNotFoundError {
    pub fn username(username: &Username) -> Box<Self> {
        Box::new(Self::Username(username.clone()))
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

        write!(f, "user not found where: {}", missing)
    }
}

impl std::error::Error for UserNotFoundError {
}

/// A username the caller tried to claim is already taken (OXA-000050).
///
/// The `users_username_key` unique index is the sole arbiter of username
/// uniqueness — there is no check-then-insert — so the persistence boundary
/// translates its SQLSTATE `23505` verdict into this type.
///
/// `Display` is the sanitized wire copy: it reaches the HTTP body of the
/// unauthenticated `POST /auth/register` through `IntoOxidAuthError::into_error`
/// (crate::error), so it names neither the SQLSTATE nor the constraint and never
/// echoes the username. The username stays on the struct for `Debug` / `tracing`.
#[derive(Debug)]
pub struct UserAlreadyExistsError {
    pub username: Username,
}

impl UserAlreadyExistsError {
    pub fn username(username: &Username) -> Box<Self> {
        Box::new(Self {
            username: username.clone(),
        })
    }
}

impl fmt::Display for UserAlreadyExistsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("username is already taken")
    }
}

impl std::error::Error for UserAlreadyExistsError {
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn user_kind_round_trips_through_str_and_serde() {
        let human: &'static str = (&UserKind::Human).into();
        assert_eq!(human, "human");

        let api: &'static str = (&UserKind::Api).into();
        assert_eq!(api, "api");

        assert_eq!(
            HUMAN
                .parse::<UserKind>()
                .unwrap(),
            UserKind::Human
        );
        assert_eq!(
            API.parse::<UserKind>()
                .unwrap(),
            UserKind::Api
        );

        assert_eq!(UserKind::default(), UserKind::Human);

        // the DB / JSON wire stores the exact snake_case token
        assert_eq!(
            serde_json::to_value(UserKind::Human).unwrap(),
            json!("human")
        );
        assert_eq!(serde_json::to_value(UserKind::Api).unwrap(), json!("api"));
        assert_eq!(
            serde_json::from_value::<UserKind>(json!("api")).unwrap(),
            UserKind::Api
        );
    }

    #[test]
    fn user_kind_rejects_unknown_strings() {
        let err = "robot"
            .parse::<UserKind>()
            .unwrap_err();
        assert_eq!(err.to_string(), "failed to parse user_kind, unknown: robot");

        // matching is case- and whitespace-sensitive
        assert!(
            "Human"
                .parse::<UserKind>()
                .is_err()
        );
        assert!(
            " api"
                .parse::<UserKind>()
                .is_err()
        );
        assert!(
            "".parse::<UserKind>()
                .is_err()
        );
    }

    #[test]
    fn user_status_round_trips_through_str_and_serde() {
        let enabled: &'static str = (&UserStatus::Enabled).into();
        assert_eq!(enabled, "enabled");

        let invited: &'static str = (&UserStatus::Invited).into();
        assert_eq!(invited, "invited");

        let disabled: &'static str = (&UserStatus::Disabled).into();
        assert_eq!(disabled, "disabled");

        assert!(matches!(
            ENABLED
                .parse::<UserStatus>()
                .unwrap(),
            UserStatus::Enabled
        ));
        assert!(matches!(
            INVITED
                .parse::<UserStatus>()
                .unwrap(),
            UserStatus::Invited
        ));
        assert!(matches!(
            DISABLED
                .parse::<UserStatus>()
                .unwrap(),
            UserStatus::Disabled
        ));

        assert!(matches!(UserStatus::default(), UserStatus::Enabled));

        assert_eq!(
            serde_json::to_value(UserStatus::Invited).unwrap(),
            json!("invited")
        );
        assert!(matches!(
            serde_json::from_value::<UserStatus>(json!("disabled")).unwrap(),
            UserStatus::Disabled
        ));
    }

    #[test]
    fn user_status_rejects_unknown_strings() {
        let err = "paused"
            .parse::<UserStatus>()
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            "failed to parse user_status, unknown: paused"
        );

        assert!(
            "Enabled"
                .parse::<UserStatus>()
                .is_err()
        );
        assert!(
            "".parse::<UserStatus>()
                .is_err()
        );
    }

    #[test]
    fn username_display_and_from_str_round_trip() {
        let username: Username = "root".parse().unwrap();

        assert_eq!(username.to_string(), "root");
        assert_eq!(username.0, "root");

        // `Username::FromStr` is infallible — any text is a username as far as
        // the kernel is concerned (validation lives elsewhere)
        assert_eq!(
            "".parse::<Username>()
                .unwrap()
                .to_string(),
            ""
        );
    }

    #[test]
    fn user_already_exists_error_sanitizes_the_wire_copy_but_keeps_the_username_for_debug() {
        let err = UserAlreadyExistsError::username(&"root".parse().unwrap());

        // `Display` is the copy `into_error` puts in the HTTP body of an
        // unauthenticated endpoint: no username, no SQLSTATE, no constraint.
        assert_eq!(err.to_string(), "username is already taken");

        // `Debug` (the `$.errors[0].debug` field + `tracing`) keeps the type
        // name — the hurl suites match on it — and the offending username.
        let debug = format!("{err:?}");
        assert!(debug.contains("UserAlreadyExistsError"), "{debug}");
        assert!(debug.contains("root"), "{debug}");
        assert_eq!(err.username.0, "root");
    }
}
