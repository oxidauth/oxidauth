use std::{fmt, ops::Deref};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod auth;
pub mod authorities;
pub mod bootstrap;
pub mod dev_prelude;
pub mod error;
pub mod invitations;
pub mod jwt;
pub mod permissions;
pub mod prelude;
pub mod private_keys;
pub mod provider;
pub mod public_keys;
pub mod refresh_tokens;
pub mod role_permission_grants;
pub mod role_role_grants;
pub mod roles;
pub mod rsa;
pub mod service;
pub mod settings;
pub mod user_authorities;
pub mod user_permission_grants;
pub mod user_role_grants;
pub mod users;

pub mod base64 {
    pub use base64::prelude::*;
}

#[derive(Clone, Deserialize, Serialize)]
pub struct JsonValue(Value);

impl Deref for JsonValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl JsonValue {
    pub fn new(json: Value) -> Self {
        Self(json)
    }

    pub fn inner_value(self) -> Value {
        self.0
    }
}

impl fmt::Debug for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsonValue")
            .finish()
    }
}

impl From<Value> for JsonValue {
    fn from(value: Value) -> Self {
        JsonValue(value)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Password(String);

impl Password {
    pub fn new(password: String) -> Self {
        Self(password)
    }

    pub fn inner_value(self) -> String {
        self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res: String = self
            .0
            .chars()
            .enumerate()
            .map(|(i, ch)| {
                if i == 0 {
                    ch
                } else {
                    '*'
                }
            })
            .collect();

        f.debug_struct(&res).finish()
    }
}

impl From<String> for Password {
    fn from(value: String) -> Self {
        Self(value)
    }
}
