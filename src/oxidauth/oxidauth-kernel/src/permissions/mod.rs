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

#[derive(Debug)]
pub struct PermissionNotFoundError {
    permission: String,
}

impl PermissionNotFoundError {
    pub fn new(permission: &str) -> Box<Self> {
        Box::new(Self {
            permission: permission.to_owned(),
        })
    }
}

impl fmt::Display for PermissionNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "permission with name not found: {}", self.permission)
    }
}

impl std::error::Error for PermissionNotFoundError {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn permission(realm: &str, resource: &str, action: &str) -> Permission {
        Permission {
            id: Uuid::new_v4(),
            realm: realm.to_owned(),
            resource: resource.to_owned(),
            action: action.to_owned(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn raw_permission_parses_valid_three_part_strings() {
        let raw = RawPermission::try_from("oxidauth:users:read").unwrap();

        assert_eq!(raw.realm, "oxidauth");
        assert_eq!(raw.resource, "users");
        assert_eq!(raw.action, "read");

        // wildcards are just non-empty segments for this wrapper (the
        // oxidauth-permission crate owns wildcard grammar)
        let raw = RawPermission::try_from("oxidauth:**:**").unwrap();

        assert_eq!(raw.realm, "oxidauth");
        assert_eq!(raw.action, "**");
    }

    #[test]
    fn raw_permission_rejects_missing_or_empty_parts() {
        // fewer than three segments
        let err = RawPermission::try_from("oxidauth:users").unwrap_err();
        assert_eq!(
            err,
            "a permission must have all three parts: 'oxidauth:users'"
        );

        let err = RawPermission::try_from("oxidauth").unwrap_err();
        assert_eq!(err, "a permission must have all three parts: 'oxidauth'");

        let err = RawPermission::try_from("").unwrap_err();
        assert_eq!(err, "a permission must have all three parts: ''");

        // an empty segment in any of the first three positions is rejected with
        // the same message
        for empty in [":users:read", "oxidauth::read", "oxidauth:users:"] {
            let err = RawPermission::try_from(empty).unwrap_err();
            assert_eq!(
                err,
                format!("a permission must have all three parts: '{empty}'")
            );
        }
    }

    #[test]
    fn raw_permission_silently_drops_parts_beyond_the_third() {
        // BUG(pinned): only the first three segments are validated, the rest
        // are ignored — "a:b:c:d" yields action == "c" even though the
        // oxidauth-permission crate's own parser rejects four-part strings.
        let raw = RawPermission::try_from("a:b:c:d").unwrap();

        assert_eq!(raw.realm, "a");
        assert_eq!(raw.resource, "b");
        assert_eq!(raw.action, "c");
    }

    #[test]
    fn all_try_from_paths_agree() {
        let owned = "oxidauth:users:read".to_string();

        let from_str = RawPermission::try_from(owned.as_str()).unwrap();
        let from_ref = RawPermission::try_from(&owned).unwrap();
        let from_val = RawPermission::try_from(owned.clone()).unwrap();

        assert_eq!(from_str, from_ref);
        assert_eq!(from_ref, from_val);

        let bad = "oxidauth".to_string();

        let err_str = RawPermission::try_from(bad.as_str()).unwrap_err();
        let err_ref = RawPermission::try_from(&bad).unwrap_err();
        let err_val = RawPermission::try_from(bad.clone()).unwrap_err();

        assert_eq!(err_str, err_ref);
        assert_eq!(err_ref, err_val);
    }

    #[test]
    fn permission_displays_as_the_colon_triple() {
        // Permission itself has no constructor (fields are public); Display is
        // what feeds entitlement strings in the permission tree
        let permission = permission("oxidauth", "users", "read");

        assert_eq!(permission.to_string(), "oxidauth:users:read");
    }
}
