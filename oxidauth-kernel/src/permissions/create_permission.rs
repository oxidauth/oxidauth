use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Permission;

pub type CreatePermissionService = Arc<
    dyn for<'a> Service<
        &'a str,
        Response = Permission,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct CreatePermission {
    pub id: Option<Uuid>,
    pub realm: String,
    pub resource: String,
    pub action: String,
}

impl<'a> TryFrom<&'a str> for CreatePermission {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let parts: Vec<&'a str> = value.split(":").collect();

        if parts.len() < 3 {
            return Err(format!(
                "a permission must have all three parts: '{}'",
                value
            ));
        }

        for field in parts[0..3].iter() {
            if field.len() == 0 {
                return Err(format!(
                    "a permission must have all three parts: '{}'",
                    value
                ));
            }
        }

        Ok(CreatePermission {
            id: None,
            realm: parts[0].to_owned(),
            resource: parts[1].to_owned(),
            action: parts[2].to_owned(),
        })
    }
}

impl<'a> TryFrom<&'a String> for CreatePermission {
    type Error = String;

    fn try_from(value: &'a String) -> Result<Self, Self::Error> {
        let value: &'a str = value.as_ref();

        value.try_into()
    }
}

impl TryFrom<String> for CreatePermission {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value: &str = value.as_ref();

        value.try_into()
    }
}
