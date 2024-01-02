use core::fmt;

use chrono::{DateTime, Utc};
use oxidauth_kernel::users::{ParseUserKindErr, ParseUserStatusErr, User};
use serde_json::Value;
use uuid::Uuid;

pub mod insert_user;

#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub kind: String,
    pub status: String,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = TryFromUserRowError;

    fn try_from(row: UserRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            kind: row.kind.parse()?,
            status: row.status.parse()?,
            username: row.username,
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            profile: row.profile,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug)]
pub enum TryFromUserRowError {
    ParseUserKindErr(ParseUserKindErr),
    ParseUserStatusErr(ParseUserStatusErr),
}

impl fmt::Display for TryFromUserRowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TryFromUserRowError::ParseUserKindErr(err) => err.to_string(),
            TryFromUserRowError::ParseUserStatusErr(err) => err.to_string(),
        };

        write!(
            f,
            "failed to convert UserRow to User: {}",
            s
        )
    }
}

impl std::error::Error for TryFromUserRowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            TryFromUserRowError::ParseUserKindErr(ref err) => Some(err),
            TryFromUserRowError::ParseUserStatusErr(ref err) => Some(err),
        }
    }
}

impl From<ParseUserKindErr> for TryFromUserRowError {
    fn from(err: ParseUserKindErr) -> Self {
        TryFromUserRowError::ParseUserKindErr(err)
    }
}

impl From<ParseUserStatusErr> for TryFromUserRowError {
    fn from(err: ParseUserStatusErr) -> Self {
        TryFromUserRowError::ParseUserStatusErr(err)
    }
}
