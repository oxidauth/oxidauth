use std::{error::Error, fmt, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};
use url::Url;

pub mod create_authority;
pub mod delete_authority;
pub mod find_authority_by_client_key;
pub mod find_authority_by_id;
pub mod find_authority_by_strategy;
pub mod list_all_authorities;
pub mod update_authority;

pub use crate::user_authorities::UserAuthority;
use crate::{JsonValue, dev_prelude::*, jwt::EntitlementsEncoding};

#[derive(Debug, Serialize, Deserialize)]
pub struct Authority {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: AuthorityStatus,
    pub strategy: AuthorityStrategy,
    pub settings: AuthoritySettings,
    pub params: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthoritySettings {
    pub jwt_ttl: Duration,
    pub jwt_nbf_offset: NbfOffset,
    pub refresh_token_ttl: Duration,
    pub totp: TotpSettings,
    /// Wire encoding of the `entitlements` claim (`txt` / `gz`). Note the
    /// size caveat: `gz` gzip+base64-wraps the permission list, and
    /// base64 expands ~4/3 before compression nets out — for typical
    /// short permission lists the `gz` claim is *larger* than `txt`;
    /// `gz` only pays off for very large entitlement sets (OXA-000040).
    pub entitlements_encoding: EntitlementsEncoding,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NbfOffset {
    Enabled(Duration),
    Disabled,
}

pub const DEFAULT_JWT_NBF: Duration = Duration::from_secs(10);

impl Default for NbfOffset {
    fn default() -> Self {
        Self::Enabled(DEFAULT_JWT_NBF)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TotpSettings {
    // Aliases keep the pre-OXA-000033 PascalCase tags decoding forever:
    // stored `authorities.settings` JSONB rows and old SDK clients are
    // alias-dependent (OXA-000033). Emission is snake_case only.
    #[serde(alias = "Enabled")]
    Enabled {
        totp_ttl: Duration,
        webhook: Url,
        webhook_key: String,
    },
    #[serde(alias = "Disabled")]
    Disabled,
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
            status => Err(format!("invalid authority status: {}", status).into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityStrategy {
    UsernamePassword,
    SingleUseToken,
    Oauth2,
}

impl AuthorityStrategy {
}

const USERNAME_PASSWORD: &str = "username_password";
const SINGLE_USE_TOKEN: &str = "single_use_token";
const OAUTH2: &str = "oauth2";

impl fmt::Display for AuthorityStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AuthorityStrategy::*;

        match self {
            UsernamePassword => write!(f, "{}", USERNAME_PASSWORD),
            SingleUseToken => write!(f, "{}", SINGLE_USE_TOKEN),
            Oauth2 => write!(f, "{}", OAUTH2),
        }
    }
}

impl FromStr for AuthorityStrategy {
    type Err = ParseAuthorityStrategyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            USERNAME_PASSWORD => AuthorityStrategy::UsernamePassword,
            SINGLE_USE_TOKEN => AuthorityStrategy::SingleUseToken,
            OAUTH2 => AuthorityStrategy::Oauth2,
            strategy => {
                return Err(ParseAuthorityStrategyError::Unknown(strategy.to_owned()));
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
            Unknown(value) => write!(f, "unknown authority strategy: {}", value),
        }
    }
}

impl Error for ParseAuthorityStrategyError {
}

#[derive(Debug)]
pub enum AuthorityNotFoundError {
    Strategy(AuthorityStrategy),
    Id(Uuid),
    ClientKey(Uuid),
}

impl AuthorityNotFoundError {
    pub fn strategy(strategy: AuthorityStrategy) -> Box<Self> {
        Box::new(Self::Strategy(strategy))
    }

    pub fn id(id: Uuid) -> Box<Self> {
        Box::new(Self::Id(id))
    }

    pub fn client_key(id: Uuid) -> Box<Self> {
        Box::new(Self::ClientKey(id))
    }
}

impl fmt::Display for AuthorityNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthorityNotFoundError::Strategy(strategy) => {
                write!(f, "authority not found by strategy: {}", strategy)
            },
            AuthorityNotFoundError::Id(id) => write!(f, "authority not found by id: {}", id,),
            AuthorityNotFoundError::ClientKey(id) => {
                write!(f, "authority not found by client_key: {}", id,)
            },
        }
    }
}

impl Error for AuthorityNotFoundError {
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn authority_strategy_round_trips_through_display_and_from_str() {
        let cases = [
            (AuthorityStrategy::UsernamePassword, "username_password"),
            (AuthorityStrategy::SingleUseToken, "single_use_token"),
            (AuthorityStrategy::Oauth2, "oauth2"),
        ];

        for (strategy, expected) in cases {
            assert_eq!(strategy.to_string(), expected);

            // Display -> FromStr is the loop the DB text columns ride through;
            // the enums are compared via Debug as they don't implement PartialEq
            let parsed = expected
                .parse::<AuthorityStrategy>()
                .unwrap();
            assert_eq!(format!("{parsed:?}"), format!("{strategy:?}"));

            // the JSON wire uses the same snake_case token
            assert_eq!(serde_json::to_value(strategy).unwrap(), json!(expected));

            let from_json = serde_json::from_value::<AuthorityStrategy>(json!(expected)).unwrap();
            assert_eq!(format!("{from_json:?}"), format!("{strategy:?}"));
        }
    }

    #[test]
    fn authority_strategy_rejects_unknown_strings() {
        let err = "ldap"
            .parse::<AuthorityStrategy>()
            .unwrap_err();
        assert_eq!(err.to_string(), "unknown authority strategy: ldap");

        // near-misses: wrong separator, wrong case, empty
        assert!(
            "usernamepassword"
                .parse::<AuthorityStrategy>()
                .is_err()
        );
        assert!(
            "UsernamePassword"
                .parse::<AuthorityStrategy>()
                .is_err()
        );
        assert!(
            "".parse::<AuthorityStrategy>()
                .is_err()
        );
    }

    #[test]
    fn authority_status_round_trips_through_display_and_from_str() {
        assert_eq!(AuthorityStatus::Enabled.to_string(), "enabled");
        assert_eq!(AuthorityStatus::Disabled.to_string(), "disabled");

        assert!(matches!(
            "enabled"
                .parse::<AuthorityStatus>()
                .unwrap(),
            AuthorityStatus::Enabled
        ));
        assert!(matches!(
            "disabled"
                .parse::<AuthorityStatus>()
                .unwrap(),
            AuthorityStatus::Disabled
        ));

        assert_eq!(
            serde_json::to_value(AuthorityStatus::Enabled).unwrap(),
            json!("enabled")
        );
        assert_eq!(
            serde_json::to_value(AuthorityStatus::Disabled).unwrap(),
            json!("disabled")
        );
    }

    #[test]
    fn authority_status_rejects_unknown_strings() {
        let err = "paused"
            .parse::<AuthorityStatus>()
            .unwrap_err();
        assert_eq!(err.to_string(), "invalid authority status: paused");

        assert!(
            "Enabled"
                .parse::<AuthorityStatus>()
                .is_err()
        );
        assert!(
            "".parse::<AuthorityStatus>()
                .is_err()
        );
    }

    #[test]
    fn nbf_offset_defaults_to_enabled_with_the_kernel_default() {
        assert!(matches!(
            NbfOffset::default(),
            NbfOffset::Enabled(d) if d == DEFAULT_JWT_NBF
        ));
        assert_eq!(DEFAULT_JWT_NBF, std::time::Duration::from_secs(10));
    }

    #[test]
    fn nbf_offset_serde_wire_shape_is_pinned() {
        // AuthoritySettings.jwt_nbf_offset rides DB/API JSON as an
        // externally-tagged snake_case enum, and `std::time::Duration` uses
        // serde's canonical `{secs, nanos}` struct form — NOT bare seconds.
        // A2's `insert_authority` settings round-trip asserts against this.
        assert_eq!(
            serde_json::to_value(NbfOffset::Enabled(std::time::Duration::from_secs(10))).unwrap(),
            json!({"enabled": {"secs": 10, "nanos": 0}})
        );

        assert_eq!(
            serde_json::to_value(NbfOffset::Enabled(std::time::Duration::from_millis(1500)))
                .unwrap(),
            json!({"enabled": {"secs": 1, "nanos": 500_000_000}})
        );

        assert_eq!(
            serde_json::to_value(NbfOffset::Disabled).unwrap(),
            json!("disabled")
        );

        // from_value closes the loop for every arm
        let enabled: NbfOffset =
            serde_json::from_value(json!({"enabled": {"secs": 10, "nanos": 0}})).unwrap();
        assert!(matches!(
            enabled,
            NbfOffset::Enabled(d) if d == std::time::Duration::from_secs(10)
        ));

        let disabled: NbfOffset = serde_json::from_value(json!("disabled")).unwrap();
        assert!(matches!(disabled, NbfOffset::Disabled));

        // default round-trips through the wire unchanged
        let round_trip: NbfOffset =
            serde_json::from_value(serde_json::to_value(NbfOffset::default()).unwrap()).unwrap();
        assert!(matches!(round_trip, NbfOffset::Enabled(d) if d == DEFAULT_JWT_NBF));
    }

    #[test]
    fn totp_settings_serde_wire_shape_is_pinned() {
        // OXA-000033: AuthoritySettings.totp rides DB/API JSON as an
        // externally-tagged snake_case enum, like NbfOffset/AuthorityStatus/
        // AuthorityStrategy. `std::time::Duration` uses serde's canonical
        // `{secs, nanos}` struct form. The pre-rename PascalCase tags must
        // keep decoding forever via serde aliases: stored JSONB rows and
        // old SDK clients are alias-dependent.
        assert_eq!(
            serde_json::to_value(TotpSettings::Disabled).unwrap(),
            json!("disabled")
        );

        let enabled = TotpSettings::Enabled {
            totp_ttl: std::time::Duration::from_secs(90),
            webhook: "https://hooks.example.com/otp"
                .parse()
                .unwrap(),
            webhook_key: "s3cret".to_string(),
        };
        assert_eq!(
            serde_json::to_value(&enabled).unwrap(),
            json!({
                "enabled": {
                    "totp_ttl": { "secs": 90, "nanos": 0 },
                    "webhook": "https://hooks.example.com/otp",
                    "webhook_key": "s3cret"
                }
            })
        );

        // from_value closes the loop on the snake_case forms
        let disabled: TotpSettings = serde_json::from_value(json!("disabled")).unwrap();
        assert!(matches!(disabled, TotpSettings::Disabled));

        let enabled: TotpSettings = serde_json::from_value(json!({
            "enabled": {
                "totp_ttl": { "secs": 90, "nanos": 0 },
                "webhook": "https://hooks.example.com/otp",
                "webhook_key": "s3cret"
            }
        }))
        .unwrap();
        assert!(matches!(enabled, TotpSettings::Enabled { .. }));

        // dual-accept: the legacy PascalCase tags decode via aliases
        let disabled: TotpSettings = serde_json::from_value(json!("Disabled")).unwrap();
        assert!(matches!(disabled, TotpSettings::Disabled));

        let enabled: TotpSettings = serde_json::from_value(json!({
            "Enabled": {
                "totp_ttl": { "secs": 90, "nanos": 0 },
                "webhook": "https://hooks.example.com/otp",
                "webhook_key": "s3cret"
            }
        }))
        .unwrap();
        assert!(matches!(enabled, TotpSettings::Enabled { .. }));
    }
}
