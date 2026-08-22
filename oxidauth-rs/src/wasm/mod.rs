pub mod builder;

use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

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
                key,
                value,
                err.to_string()
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
