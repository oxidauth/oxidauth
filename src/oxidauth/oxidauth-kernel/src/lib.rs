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
pub mod public_keys;
pub mod refresh_tokens;
pub mod role_permission_grants;
pub mod role_role_grants;
pub mod roles;
pub mod rsa;
pub mod settings;
pub mod totp;
pub mod totp_secrets;
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

    pub fn empty() -> Self {
        Self(Value::Null)
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

/// A password value; the raw material is reachable only via
/// [`Password::inner_value`].
///
/// This type deliberately does **not** implement `Serialize` (OXA-000008):
/// a derived newtype `Serialize` emitted the secret verbatim with no signal.
/// When a wire payload genuinely needs the raw value, build `json!({ ... })`
/// from an explicit `String` at the site that must carry it — precedent:
/// `oxidauth-rs`'s `Client::auth` authenticate body.
#[derive(Clone, Deserialize)]
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
        // `Debug` is what tracing spans record, so the mask stays constant:
        // neither the length nor the first character may leak
        f.debug_struct("******")
            .finish()
    }
}

impl From<String> for Password {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const SECRET: &str = "correct horse battery staple";

    #[test]
    fn password_debug_redacts_the_secret() {
        let password = Password::new(SECRET.to_string());

        let debug = format!("{password:?}");

        // tracing spans record `Debug`, so the constant mask must not leak the
        // value, its length, or any prefix of it
        assert_eq!(debug, "******");
        assert!(!debug.contains(SECRET));
        assert!(!debug.contains("correct"));
    }

    #[test]
    fn password_round_trips_through_new_from_and_inner_value() {
        assert_eq!(Password::new(SECRET.to_string()).inner_value(), SECRET);

        let from_string: Password = SECRET.to_string().into();
        assert_eq!(from_string.inner_value(), SECRET);
    }

    #[test]
    fn password_serde_deserializes_but_never_serializes() {
        // `Deserialize` stays: every inbound parse goes through it
        // (`TryFrom<JsonValue>` in the username_password authenticator and
        // registrar). `Serialize` was removed in OXA-000008 — the derived
        // newtype impl carried the raw secret verbatim.
        assert_eq!(
            serde_json::from_value::<Password>(json!(SECRET))
                .unwrap()
                .inner_value(),
            SECRET
        );

        // Red pin that `Password: !Serialize`: the inherent
        // `is_serialize_impl` is applicable only when the bound holds, so
        // re-deriving `Serialize` resolves the call to it and this test fails.
        // The `JsonValue` leg validates the probe itself detects a `Serialize`
        // type rather than always reporting `false`.
        struct SerializeProbe<T: ?Sized>(std::marker::PhantomData<T>);

        trait NotSerialize {
            fn is_serialize_impl() -> bool {
                false
            }
        }
        impl<T: ?Sized> NotSerialize for SerializeProbe<T> {
        }

        impl<T: serde::Serialize + ?Sized> SerializeProbe<T> {
            fn is_serialize_impl() -> bool {
                true
            }
        }

        assert!(
            !SerializeProbe::<Password>::is_serialize_impl(),
            "`Password` must not implement `Serialize` — it would carry the raw secret"
        );
        assert!(
            SerializeProbe::<JsonValue>::is_serialize_impl(),
            "the probe must detect a type that does implement `Serialize`"
        );
    }

    #[test]
    fn json_value_debug_does_not_leak_nested_content() {
        let json = JsonValue::new(json!({"password": SECRET}));

        let debug = format!("{json:?}");

        assert_eq!(debug, "JsonValue");
        assert!(!debug.contains(SECRET));
    }

    #[test]
    fn json_value_inner_value_deref_and_from_round_trip() {
        let value = json!({"a": 1});

        let from_value: JsonValue = value.clone().into();

        // `Deref` exposes the underlying `Value` for indexing
        assert_eq!(from_value["a"], json!(1));

        assert_eq!(serde_json::to_value(&from_value).unwrap(), value);

        assert_eq!(from_value.inner_value(), value);

        assert_eq!(JsonValue::new(value.clone()).inner_value(), value);
        assert!(
            JsonValue::empty()
                .inner_value()
                .is_null()
        );
        assert_eq!(
            serde_json::from_value::<JsonValue>(value.clone())
                .unwrap()
                .inner_value(),
            value
        );
    }
}
