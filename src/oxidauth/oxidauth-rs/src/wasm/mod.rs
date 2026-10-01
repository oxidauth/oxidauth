pub mod builder;

use std::{fmt, ops::Deref, str::FromStr, sync::Arc};

use gloo_storage::{LocalStorage, Storage as _};
use tokio::sync::Mutex;

pub(crate) const JWT_KEY: &str = "OXIDAUTH_JWT";
pub(crate) const REFRESH_TOKEN_KEY: &str = "OXIDAUTH_REFRESH_TOKEN";
pub(crate) const PUBLIC_KEYS_KEY: &str = "OXIDAUTH_PUBLIC_KEYS";

pub struct Config {
    pub public_keys_ttl: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            public_keys_ttl: 120,
        }
    }
}

#[derive(Clone)]
pub struct OxidauthClient {
    pub inner: Arc<Inner>,
}

#[derive(Default)]
pub struct Inner {
    pub host: String,
    pub config: Config,
    pub state: Arc<Mutex<State>>,
}

impl Deref for OxidauthClient {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.inner
    }
}

impl OxidauthClient {
    pub fn new(host: String, config: Config) -> Self {
        let mut state = State::default();

        state.load();

        Self {
            inner: Arc::new(Inner {
                host,
                config,
                state: Arc::new(Mutex::new(state)),
            }),
        }
    }

    pub fn builder() -> builder::OxidauthClientBuilder {
        builder::OxidauthClientBuilder::new()
    }

    pub async fn clear_state(&self) {
        self.inner
            .state
            .lock()
            .await
            .clear();
    }
}

#[derive(Default)]
pub struct State {
    pub jwt: Option<String>,
    pub refresh_token: Option<String>,
    pub public_keys: Option<String>,
}

impl State {
    pub fn set(&mut self, key: StateKey, value: Option<String>) -> Result<(), String> {
        let result = match key {
            StateKey::Jwt => {
                let result = LocalStorage::set(JWT_KEY, value.as_deref());

                if result.is_ok() {
                    self.jwt = value.clone();
                }

                result
            },
            StateKey::RefreshToken => {
                let result = LocalStorage::set(REFRESH_TOKEN_KEY, value.as_deref());

                if result.is_ok() {
                    self.refresh_token = value.clone();
                }

                result
            },
            StateKey::PublicKeys => {
                let result = LocalStorage::set(PUBLIC_KEYS_KEY, value.as_deref());

                if result.is_ok() {
                    self.public_keys = value.clone();
                }

                result
            },
        };

        result.map_err(|err| {
            format!(
                "error saving to LocalStorage: key: {}, value: {:?}, err: {}",
                key, value, err
            )
        })
    }

    pub fn get(&self, key: StateKey) -> Option<&str> {
        match key {
            StateKey::Jwt => self.jwt.as_deref(),
            StateKey::RefreshToken => self.refresh_token.as_deref(),
            StateKey::PublicKeys => self.public_keys.as_deref(),
        }
    }

    pub fn load(&mut self) {
        self.jwt = LocalStorage::get(JWT_KEY).ok();
        self.refresh_token = LocalStorage::get(REFRESH_TOKEN_KEY).ok();
        self.public_keys = LocalStorage::get(PUBLIC_KEYS_KEY).ok();
    }

    pub fn clear(&mut self) {
        self.jwt = None;
        self.refresh_token = None;
        self.public_keys = None;

        LocalStorage::delete(REFRESH_TOKEN_KEY);
        LocalStorage::delete(JWT_KEY);
        LocalStorage::delete(PUBLIC_KEYS_KEY);
    }
}

#[derive(Clone, Debug)]
pub enum StateKey {
    Jwt,
    RefreshToken,
    PublicKeys,
}

impl fmt::Display for StateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateKey::Jwt => write!(f, "{}", JWT_KEY),
            StateKey::RefreshToken => write!(f, "{}", REFRESH_TOKEN_KEY),
            StateKey::PublicKeys => write!(f, "{}", PUBLIC_KEYS_KEY),
        }
    }
}

impl FromStr for StateKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            JWT_KEY => Ok(StateKey::Jwt),
            REFRESH_TOKEN_KEY => Ok(StateKey::RefreshToken),
            PUBLIC_KEYS_KEY => Ok(StateKey::PublicKeys),
            _ => Err("invalid State key".to_string()),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    // E6: what the wasm module exposes natively (`cargo test -p oxidauth
    // --features wasm`) — run with --features wasm since the module itself is
    // feature-gated.
    //
    // NOT natively testable in this file: `State::set`, `State::load`,
    // `State::clear`, `OxidauthClient::new` (calls `load`) and
    // `clear_state` (calls `clear`) — every one delegates to gloo-storage's
    // LocalStorage, which panics through js-sys on non-wasm targets (pinned
    // by `client_construction_panics_on_native` below). Browser persistence
    // semantics are only observable under a wasm runner.

    #[test]
    fn config_defaults_to_a_120s_public_keys_ttl() {
        assert_eq!(Config::default().public_keys_ttl, 120);
    }

    #[test]
    fn state_key_display_and_from_str_round_trip_through_the_storage_keys() {
        for key in [StateKey::Jwt, StateKey::RefreshToken, StateKey::PublicKeys] {
            let wire = key.to_string();
            let back: StateKey = wire.parse().unwrap();
            assert_eq!(back.to_string(), wire);
        }

        assert!(matches!(
            "OXIDAUTH_JWT"
                .parse()
                .unwrap(),
            StateKey::Jwt
        ));
        assert!(matches!(
            "OXIDAUTH_REFRESH_TOKEN"
                .parse()
                .unwrap(),
            StateKey::RefreshToken
        ));
        assert!(matches!(
            "OXIDAUTH_PUBLIC_KEYS"
                .parse()
                .unwrap(),
            StateKey::PublicKeys
        ));

        assert_eq!(StateKey::Jwt.to_string(), "OXIDAUTH_JWT");
        assert_eq!(StateKey::RefreshToken.to_string(), "OXIDAUTH_REFRESH_TOKEN");
        assert_eq!(StateKey::PublicKeys.to_string(), "OXIDAUTH_PUBLIC_KEYS");

        // near-misses and empties are rejected with the fixed copy
        for bad in ["", "JWT", "oxidauth_jwt", "OXIDAUTH_", "other"] {
            assert_eq!(
                bad.parse::<StateKey>()
                    .unwrap_err(),
                "invalid State key"
            );
        }
    }

    #[test]
    fn state_get_reads_the_in_memory_fields_purely() {
        // `get` never touches LocalStorage, so it is fully testable natively
        let mut state = State::default();
        assert!(
            state
                .get(StateKey::Jwt)
                .is_none()
        );
        assert!(
            state
                .get(StateKey::RefreshToken)
                .is_none()
        );
        assert!(
            state
                .get(StateKey::PublicKeys)
                .is_none()
        );

        state.jwt = Some("jwt".to_string());
        state.refresh_token = Some("rt".to_string());
        state.public_keys = Some("[]".to_string());
        assert_eq!(state.get(StateKey::Jwt), Some("jwt"));
        assert_eq!(state.get(StateKey::RefreshToken), Some("rt"));
        assert_eq!(state.get(StateKey::PublicKeys), Some("[]"));
    }

    #[test]
    fn client_construction_panics_on_native() {
        // Pins the boundary itself: `OxidauthClient::new` runs `State::load`,
        // whose LocalStorage access is a hard panic off-wasm (js-sys). Any
        // native construction of the wasm client is therefore unsupported —
        // the builder's two validation arms are the only pure surface.
        let result = std::panic::catch_unwind(|| {
            OxidauthClient::new("https://oxidauth.test".to_string(), Config::default())
        });
        assert!(
            result.is_err(),
            "native OxidauthClient::new must panic on the missing browser storage"
        );
    }

    #[test]
    fn builder_factory_is_pure() {
        // `OxidauthClient::builder()` must not construct (and panic on) a client
        let _builder = OxidauthClient::builder();
    }
}
