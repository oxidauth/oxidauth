use std::error::Error as StdError;

use leptos::prelude::*;
use oxidauth::client::{Client, ClientError, ClientErrorKind};
use oxidauth_kernel::jwt::Jwt;
use oxidauth_permission::parse_and_validate;
use url::Url;
use uuid::Uuid;

/// Compile-time config for `trunk serve` / `trunk build`:
/// `OXIDAUTH_API_URL=http://localhost:8080 OXIDAUTH_CLIENT_KEY=<uuid> trunk serve`.
/// The client key is the uuid of the authority the console signs into — the
/// `OXIDAUTH_DEFAULT_CLIENT_KEY` value from `.env`.
pub const API_URL: &str = match option_env!("OXIDAUTH_API_URL") {
    Some(url) => url,
    None => "http://localhost:8080",
};

pub const CLIENT_KEY: &str = match option_env!("OXIDAUTH_CLIENT_KEY") {
    Some(key) => key,
    None => "",
};

/// App-wide state. The SDK client owns the whole session: it persists the
/// JWT + refresh token to LocalStorage (wasm build) and re-validates /
/// refreshes them transparently on every call — this crate never touches a
/// token directly.
#[derive(Clone, Copy)]
pub struct AppState {
    pub client: RwSignal<Client>,

    /// The decoded JWT of the live session. `None` while unknown or signed
    /// out; `Protected` only redirects once `booted` proves the check ran.
    pub session: RwSignal<Option<Jwt>>,

    /// The first session probe finished (page-load revalidation resolved).
    pub booted: RwSignal<bool>,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        let base_url = Url::parse(API_URL)
            .map_err(|err| format!("OXIDAUTH_API_URL ({API_URL}) is not a valid url: {err}"))?;

        let client_key = Uuid::parse_str(CLIENT_KEY).map_err(|_| {
            "OXIDAUTH_CLIENT_KEY is not set at build time; rebuild with \
             `OXIDAUTH_CLIENT_KEY=<authority uuid> trunk serve`"
                .to_string()
        })?;

        let client = Client::new(&base_url, client_key)
            .map_err(|err| format!("error making oxidauth client: {err}"))?;

        Ok(Self {
            client: RwSignal::new(client),
            session: RwSignal::new(None),
            booted: RwSignal::new(false),
        })
    }

    /// The current session's granted permission strings, decoded from the
    /// JWT claim (txt or gz encoding — `as_vec` handles both).
    pub fn entitlements(&self) -> Vec<String> {
        Self::entitlements_from(self.session.get())
    }

    fn entitlements_from(jwt: Option<Jwt>) -> Vec<String> {
        jwt.and_then(|jwt| {
            jwt.entitlements
                .as_ref()
                .and_then(|e| e.as_vec())
        })
        .unwrap_or_default()
    }

    /// Local RBAC against the session grants — hides affordances the server
    /// would refuse. The server stays the enforcement point: every handler
    /// re-validates the same permission-tree challenge itself.
    pub fn can(&self, challenge: &str) -> bool {
        // Event handlers and guards run outside a tracking context on
        // purpose — read the live session untracked.
        let entitlements = Self::entitlements_from(self.session.get_untracked());

        parse_and_validate(challenge, &entitlements).unwrap_or(false)
    }

    /// The reactive form of [`Self::can`], for render-time gates (`Show`,
    /// derived signals, `.then()` closures in views): derives from the
    /// session signal itself, so controls that mounted before the page-load
    /// probe landed appear the moment it does. A gate closed over
    /// [`Self::can`] would read untracked and freeze at its first verdict.
    /// Event handlers keep the untracked [`Self::can`].
    pub fn can_gate(&self, challenge: &'static str) -> Signal<bool> {
        let session = self.session;

        Signal::derive(move || {
            parse_and_validate(challenge, &Self::entitlements_from(session.get())).unwrap_or(false)
        })
    }
}

/// True when an SDK error means this session cannot produce a valid token
/// any more — signed out, or the refresh chain died. The SDK never surfaces
/// HTTP statuses (OXA-000030 closed wontfix), so session liveness keys off
/// the auth state machine's own errors, not 401s.
pub fn is_session_dead(err: &(dyn StdError + 'static)) -> bool {
    let Some(ClientError { kind, .. }) = err.downcast_ref::<ClientError>() else {
        return false;
    };

    matches!(
        kind,
        ClientErrorKind::NoJwtFound
            | ClientErrorKind::RefreshError
            | ClientErrorKind::Other("can't refresh -- no refresh token found")
            // The only producer of the empty message is refresh()'s rejected-
            // exchange arm: the refresh token was spent/unknown — the session
            // is genuinely over.
            | ClientErrorKind::Other("")
    )
}
