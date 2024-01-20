use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod create_permission;
pub mod delete_permission;
pub mod find_permission_by_parts;
pub mod list_all_permissions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Uuid,
    pub realm: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.realm, self.resource, self.action)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RawPermission {
    pub realm: String,
    pub resource: String,
    pub action: String,
}

impl<'a> TryFrom<&'a str> for RawPermission {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let parts: Vec<&'a str> = value.split(':').collect();

        if parts.len() < 3 {
            return Err(format!(
                "a permission must have all three parts: '{}'",
                value
            ));
        }

        for field in parts[0..3].iter() {
            if field.is_empty() {
                return Err(format!(
                    "a permission must have all three parts: '{}'",
                    value
                ));
            }
        }

        Ok(RawPermission {
            realm: parts[0].to_owned(),
            resource: parts[1].to_owned(),
            action: parts[2].to_owned(),
        })
    }
}

impl<'a> TryFrom<&'a String> for RawPermission {
    type Error = String;

    fn try_from(value: &'a String) -> Result<Self, Self::Error> {
        let value: &'a str = value.as_ref();

        value.try_into()
    }
}

impl TryFrom<String> for RawPermission {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value: &str = value.as_ref();

        value.try_into()
    }
}
