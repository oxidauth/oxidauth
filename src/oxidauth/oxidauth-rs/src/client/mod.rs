pub use std::fmt;
#[cfg(any(not(target_arch = "wasm32"), feature = "wasm"))]
use std::sync::Weak;
use std::{sync::Arc, time::Duration};

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::Utc;
use oxidauth_http::{
    Response,
    auth::authenticate::{AuthenticateReq, AuthenticateRes},
    public_keys::list_all_public_keys::ListAllPublicKeysRes,
    refresh_tokens::exchange::{ExchangeRefreshTokenReq, ExchangeRefreshTokenRes},
};
use oxidauth_kernel::{JsonValue, jwt::Jwt, public_keys::PublicKey};
use reqwest::{Method, header::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::info;
#[cfg(any(not(target_arch = "wasm32"), feature = "wasm"))]
use tracing::warn;
use url::Url;
use uuid::Uuid;

#[cfg(feature = "mock")]
use crate::mock::ClientMock;
pub use crate::{
    auth::AuthTrait,
    authorities::AuthoritiesTrait,
    can::CanTrait,
    client::permissions::PermissionsTrait,
    invitations::InvitationsTrait,
    public_keys::PublicKeysTrait,
    refresh_tokens::RefreshTokensTrait,
    roles::{RolesTrait, permissions::RolePermissionsTrait, roles::RoleRoleGrantsTrait},
    settings::SettingsTrait,
    users::{
        UsersTrait,
        authorities::UserAuthoritiesTrait,
        permissions::UserPermissionsTrait,
        roles::UserRolesTrait,
    },
};

pub mod auth;
pub mod authorities;
pub mod can;
pub mod invitations;
pub mod permissions;
pub mod public_keys;
pub mod refresh_tokens;
pub mod roles;
mod session;
pub mod settings;
pub mod users;

#[cfg(feature = "mock")]
pub mod mock;

// No `Send + Sync` supertraits: they only ever asserted `Client`'s own
// auto-traits, and under `wasm32-unknown-unknown` (reqwest fetch futures are
// `!Send`) they would block the impl entirely. No consumer requires them.
pub trait ClientTrait:
    AuthTrait
    + AuthoritiesTrait
    + CanTrait
    + InvitationsTrait
    + PermissionsTrait
    + PublicKeysTrait
    + RefreshTokensTrait
    + RolesTrait
    + RoleRoleGrantsTrait
    + RolePermissionsTrait
    + SettingsTrait
    + UsersTrait
    + UserAuthoritiesTrait
    + UserPermissionsTrait
    + UserRolesTrait
    + 'static
{
}

#[derive(Debug, Clone)]
pub struct Client {
    config: Config,
    state: Arc<RwLock<State>>,
    /// The single background auto-refresh schedule, shared by every clone of
    /// this client; see `Client::arm_auto_refresh`. Native builds hold the
    /// task's abort handle; wasm builds — where `spawn_local` offers no abort
    /// — hold a generation counter instead (see `Cancel`).
    #[cfg(not(target_arch = "wasm32"))]
    refresh_task: Arc<parking_lot::Mutex<Option<tokio::task::AbortHandle>>>,
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    refresh_task: Arc<AtomicUsize>,
    #[cfg(feature = "mock")]
    pub mock_jwt: Option<Jwt>,
}

impl ClientTrait for Client {
}

#[cfg(feature = "mock")]
impl ClientTrait for ClientMock {
}

/// How far ahead of a session JWT's `exp` the client rotates the pair: the
/// background auto-refresh schedule (every target) exchanges at `exp` minus
/// this window, and the request-time threshold in `check_auth_state`
/// treats a token inside the window as spent. Per-client override:
/// [`Client::with_refresh_buffer`].
pub const DEFAULT_JWT_REFRESH_BUFFER: Duration = Duration::from_secs(15);

#[derive(Debug, Clone)]
pub struct Config {
    base_url: Url,
    client_key: Uuid,
    refresh_buffer: Duration,
}

#[derive(Debug, Default)]
pub struct State {
    client: reqwest::Client,
    jwt: Option<Jwt>,
    raw_jwt: Option<String>,
    refresh_token: Option<Uuid>,
}

/// `auth()`'s strict verification. Server builds verify the fresh token
/// against the authority's public keys (`decode_with_public_keys`, pinned
/// strict contract) — the `server` feature is what enables the kernel's
/// jsonwebtoken backend. Client-only / wasm builds link no crypto backend,
/// so they parse the claims unverified instead: the browser could never
/// treat a LocalStorage token as trusted input anyway, and every api call
/// re-verifies server-side.
#[cfg(feature = "server")]
fn verify_auth_jwt(token: &str, public_keys: &[PublicKey]) -> Result<Jwt, ClientError> {
    Jwt::decode_with_public_keys(token, public_keys)
        .map_err(|_| ClientError::new(ClientErrorKind::Other("failed to validate jwt"), None))
}

#[cfg(not(feature = "server"))]
fn verify_auth_jwt(token: &str, _public_keys: &[PublicKey]) -> Result<Jwt, ClientError> {
    Jwt::decode_unverified(token)
        .map_err(|_| ClientError::new(ClientErrorKind::Other("failed to parse jwt"), None))
}

/// refresh()/recover_jwt()'s lenient verification (raw or base64 PEM keys);
/// same feature split as [`verify_auth_jwt`].
#[cfg(feature = "server")]
fn verify_session_jwt(token: &str, public_keys: &[PublicKey]) -> Result<Jwt, ClientError> {
    Jwt::decode_with_flexible_public_keys(token, public_keys)
        .map_err(|_| ClientError::new(ClientErrorKind::Other("failed to validate jwt"), None))
}

#[cfg(not(feature = "server"))]
fn verify_session_jwt(token: &str, _public_keys: &[PublicKey]) -> Result<Jwt, ClientError> {
    Jwt::decode_unverified(token)
        .map_err(|_| ClientError::new(ClientErrorKind::Other("failed to parse jwt"), None))
}

/// Builds the bearer-preloaded reqwest client every authenticated state
/// transition installs (`auth`, `refresh`, `recover_jwt`).
fn bearer_client(raw_jwt: &str) -> Result<reqwest::Client, ClientError> {
    let bearer = format!("Bearer {}", raw_jwt)
        .parse()
        .map_err(|err| {
            ClientError::new(
                ClientErrorKind::Other("unable to create bearer token"),
                Some(Box::new(err)),
            )
        })?;

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", bearer);

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|err| {
            ClientError::new(
                ClientErrorKind::Other("unable to build client in auth"),
                Some(Box::new(err)),
            )
        })
}

/// The `GET /public_keys` behind every verification path, deliberately
/// uncached: token validation always runs against the current keyset.
async fn fetch_public_keys(base_url: &Url) -> Result<Vec<PublicKey>, ClientError> {
    let public_keys: Response<ListAllPublicKeysRes> = reqwest::Client::new()
        .get(format!("{}/public_keys", base_url))
        .send()
        .await
        .map_err(|err| {
            ClientError::new(
                ClientErrorKind::Other("unable to fetch public keys"),
                Some(Box::new(err)),
            )
        })?
        .json()
        .await
        .map_err(|err| {
            ClientError::new(
                ClientErrorKind::Other("unable to deserialize public keys"),
                Some(Box::new(err)),
            )
        })?;

    let public_keys: Vec<PublicKey> = match public_keys {
        Response {
            success: true,
            payload: Some(payload),
            ..
        } => payload.public_keys,
        _ => {
            return Err(ClientError::new(
                ClientErrorKind::Other("failed to deserialize public keys"),
                None,
            ));
        },
    };

    if public_keys.is_empty() {
        return Err(ClientError::new(
            ClientErrorKind::Other("no public keys found"),
            None,
        ));
    }

    Ok(public_keys)
}

/// The refresh-token exchange behind both `Client::refresh` and the
/// background auto-refresh task.
///
/// The state write lock is held across the whole keys-fetch + exchange
/// round-trip: that is what collapses N expiry-driven callers into a single
/// exchange (pinned by
/// `concurrent_get_jwt_after_expiry_refreshes_exactly_once`).
async fn refresh_session(config: &Config, state: &RwLock<State>) -> Result<bool, ClientError> {
    let mut state = state.write().await;

    // Multi-tab (wasm build; no-op natively): a sibling tab that refreshed
    // first rotated the pair in storage — adopt it instead of exchanging
    // the refresh token this tab already spent.
    session::hydrate(&mut state);

    let public_keys = fetch_public_keys(&config.base_url).await?;

    let Some(refresh_token) = state.refresh_token else {
        return Err(ClientError::new(
            ClientErrorKind::Other("can't refresh -- no refresh token found"),
            None,
        ));
    };

    let req = ExchangeRefreshTokenReq { refresh_token };

    let response: Response<ExchangeRefreshTokenRes> = reqwest::Client::new()
        .post(format!("{}/refresh_tokens", config.base_url))
        .json(&req)
        .send()
        .await
        .map_err(|err| {
            ClientError::new(
                ClientErrorKind::Other("unable to make request for new refresh token"),
                Some(Box::new(err)),
            )
        })?
        .json()
        .await
        .map_err(|err| {
            ClientError::new(
                ClientErrorKind::Other("unable to make request for new refresh token"),
                Some(Box::new(err)),
            )
        })?;

    match response {
        Response {
            success: true,
            payload: Some(payload),
            ..
        } => {
            let jwt = verify_session_jwt(&payload.jwt, &public_keys)?;

            state.raw_jwt = Some(payload.jwt.clone());
            state.jwt = Some(jwt);
            state.refresh_token = Some(payload.refresh_token);

            state.client = bearer_client(&payload.jwt)?;
            session::persist(&state);
        },
        _ => return Err(ClientError::new(ClientErrorKind::Other(""), None)),
    }

    Ok(true)
}

/// The auto-refresh loop's wake-up primitive: a tokio timer natively, the
/// browser's `setTimeout` (`gloo-timers`) on wasm.
///
/// The wasm clamp: `setTimeout` counts `u32` milliseconds (~49.7 days max).
/// Waking early is harmless — the loop recomputes the lead from wall-clock
/// `exp` after every wake and sleeps again, so a clamped lead self-corrects.
#[cfg(not(target_arch = "wasm32"))]
async fn sleep_until_refresh(lead: Duration) {
    tokio::time::sleep(lead).await;
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
async fn sleep_until_refresh(lead: Duration) {
    let ms = lead.as_millis().min(u32::MAX as u128) as u32;

    gloo_timers::future::TimeoutFuture::new(ms).await;
}

/// How the auto-refresh loop learns it no longer owns the session schedule.
///
/// Natively `arm`/`disarm` abort the task outright — `tokio::spawn` hands back
/// a `JoinHandle` — so the loop never checks. Wasm's `spawn_local` has no
/// abort handle at all: `arm`/`disarm` bump a shared generation counter
/// instead and the loop retires when the slot no longer matches the
/// generation it was armed with.
#[cfg(any(not(target_arch = "wasm32"), feature = "wasm"))]
#[derive(Clone)]
enum Cancel {
    #[cfg(not(target_arch = "wasm32"))]
    AbortedExternally,

    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    Generation {
        slot: Arc<AtomicUsize>,
        generation: usize,
    },
}

#[cfg(any(not(target_arch = "wasm32"), feature = "wasm"))]
impl Cancel {
    fn is_stale(&self) -> bool {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::AbortedExternally => false,

            #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
            Self::Generation { slot, generation } => {
                slot.load(Ordering::Relaxed) != *generation
            },
        }
    }
}
/// The body of the background auto-refresh task: sleep until the session
/// JWT enters the refresh buffer, exchange the refresh token, and
/// reschedule against the rotated `exp` — until the session ends, an
/// exchange fails, the schedule is superseded (`cancel.is_stale()` — wasm
/// only), or every `Client` handle is dropped (the weak state can no longer
/// be upgraded).
///
/// A failed exchange retires the task on purpose: a spent/revoked refresh
/// token will never succeed, and a transient failure is covered by the
/// request-time threshold on the next call. Retrying here would hammer the
/// api on a dead session.
#[cfg(any(not(target_arch = "wasm32"), feature = "wasm"))]
async fn auto_refresh_task(config: Config, state: Weak<RwLock<State>>, cancel: Cancel) {
    loop {
        if cancel.is_stale() {
            return;
        }

        let Some(owner) = state.upgrade() else {
            return;
        };

        let waited_exp = {
            let state = owner.read().await;

            match (&state.jwt, &state.refresh_token) {
                (Some(jwt), Some(_)) => jwt.exp,
                _ => return, // logged out, or never authenticated
            }
        };
        drop(owner);

        let lead = waited_exp as i64
            - config
                .refresh_buffer
                .as_secs() as i64
            - Utc::now().timestamp();

        if lead <= 0 {
            // The token is already inside the buffer: a caller-driven
            // rotation will handle it (every session transition re-arms
            // this task). Sleeping a zero lead here would hot-loop
            // exchanges against the api.
            warn!(
                message = "oxidauth client: auto-refresh skipped, token already inside the refresh buffer",
                exp = waited_exp,
                refresh_buffer_secs = config
                    .refresh_buffer
                    .as_secs(),
            );
            return;
        }

        sleep_until_refresh(Duration::from_secs(lead as u64)).await;

        // Disarmed or re-armed during the sleep (logout, or a request-time
        // rotation that re-armed): the current schedule owns the session now.
        if cancel.is_stale() {
            return;
        }

        let Some(owner) = state.upgrade() else {
            return;
        };

        {
            let state = owner.read().await;

            // The session rotated during the sleep without this task being
            // aborted (the transition's re-arm raced the wake-up): that
            // schedule owns the rotation now, reschedule for the new token.
            match (&state.jwt, &state.refresh_token) {
                (Some(jwt), Some(_)) if jwt.exp == waited_exp => {},
                _ => continue,
            }
        }

        match refresh_session(&config, &owner).await {
            Ok(_) => info!(message = "oxidauth client: jwt auto-refreshed"),
            Err(err) => {
                warn!(
                    message = "oxidauth client: jwt auto-refresh failed, request-time refresh takes over",
                    error = %err,
                );
                return;
            },
        }
    }
}

impl Client {
    pub fn new(base_url: &Url, client_key: Uuid) -> Result<Self, ClientError> {
        let base_url = base_url
            .join("/api/v1")
            .map_err(|err| ClientError::new(ClientErrorKind::UrlParseError, Some(Box::new(err))))?;

        #[cfg(feature = "mock")]
        return Ok(Self {
            config: Config {
                base_url,
                client_key,
                refresh_buffer: DEFAULT_JWT_REFRESH_BUFFER,
            },
            state: Arc::new(RwLock::new(State::default())),
            #[cfg(any(not(target_arch = "wasm32"), feature = "wasm"))]
            refresh_task: Arc::default(),
            mock_jwt: None,
        });

        #[cfg(not(feature = "mock"))]
        {
            let mut state = State::default();
            // wasm build only (no-op natively): restore a session persisted
            // before a page reload; it is verified lazily by `recover_jwt`
            // on first use, never trusted blindly.
            session::hydrate(&mut state);

            Ok(Self {
                config: Config {
                    base_url,
                    client_key,
                    refresh_buffer: DEFAULT_JWT_REFRESH_BUFFER,
                },
                state: Arc::new(RwLock::new(state)),
                #[cfg(any(not(target_arch = "wasm32"), feature = "wasm"))]
                refresh_task: Arc::default(),
            })
        }
    }

    #[cfg(feature = "mock")]
    pub fn test_client(mock_jwt: Jwt) -> Result<Self, ClientError> {
        let base_url = Url::parse("http://base_url.com/")
            .map_err(|err| ClientError::new(ClientErrorKind::UrlParseError, Some(Box::new(err))))?
            .join("/api/v1")
            .map_err(|err| ClientError::new(ClientErrorKind::UrlParseError, Some(Box::new(err))))?;

        Ok(Self {
            config: Config {
                base_url,
                client_key: Uuid::new_v4(),
                refresh_buffer: DEFAULT_JWT_REFRESH_BUFFER,
            },
            state: Arc::new(RwLock::new(State::default())),
            #[cfg(any(not(target_arch = "wasm32"), feature = "wasm"))]
            refresh_task: Arc::default(),
            mock_jwt: Some(mock_jwt),
        })
    }

    /// Overrides [`DEFAULT_JWT_REFRESH_BUFFER`] for this client: the lead
    /// time before `exp` at which the session rotates — the background
    /// auto-refresh schedule's wake-up point (every target) and the
    /// request-time threshold in `check_auth_state`. Configure before
    /// authenticating; clones inherit the value they were cloned with.
    pub fn with_refresh_buffer(mut self, refresh_buffer: Duration) -> Self {
        self.config.refresh_buffer = refresh_buffer;
        self
    }

    pub async fn get_jwt(&self) -> Result<String, ClientError> {
        self.authenticate_if_needed()
            .await?;

        let state = self.state.read().await;

        let jwt = state
            .raw_jwt
            .as_deref()
            .ok_or(ClientError::new(ClientErrorKind::NoJwtFound, None))?;

        Ok(jwt.to_string())
    }

    pub async fn get_jwt_decoded(&self) -> Result<Jwt, ClientError> {
        self.authenticate_if_needed()
            .await?;

        let state = self.state.read().await;

        let jwt = state
            .jwt
            .clone()
            .ok_or(ClientError::new(ClientErrorKind::NoJwtFound, None))?;

        Ok(jwt)
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<bool, ClientError> {
        self.auth(username, password)
            .await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_public_keys(&self) -> Result<Vec<PublicKey>, ClientError> {
        fetch_public_keys(&self.config.base_url).await
    }

    #[tracing::instrument(skip(self))]
    async fn auth(&self, username: &str, password: &str) -> Result<bool, ClientError> {
        let mut state = self.state.write().await;

        let public_keys = self.get_public_keys().await?;

        // authenticate
        let json = AuthenticateReq {
            client_key: self.config.client_key,
            params: JsonValue::new(json!({
                "username": username,
                "password": password,
            })),
        };

        info!(message = "authenticating", params = ?json);

        let response: Response<AuthenticateRes> = reqwest::Client::new()
            .post(format!("{}/auth/authenticate", self.config.base_url))
            .json(&json)
            .send()
            .await
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::Other("unable to authenticate"),
                    Some(Box::new(err)),
                )
            })?
            .json()
            .await
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::Other("unable to deserialize authenticate"),
                    Some(Box::new(err)),
                )
            })?;

        match response {
            Response {
                success: true,
                payload: Some(payload),
                ..
            } => {
                let jwt = verify_auth_jwt(&payload.jwt, &public_keys)?;

                state.raw_jwt = Some(payload.jwt.clone());
                state.jwt = Some(jwt);
                state.refresh_token = Some(payload.refresh_token);

                state.client = bearer_client(&payload.jwt)?;
                session::persist(&state);
            },
            Response {
                success: false,
                errors: Some(errors),
                ..
            } => {
                let errors = serde_json::to_string(&errors).map_err(|err| {
                    ClientError::new(
                        ClientErrorKind::Other("unable to serialize authenticate errors"),
                        Some(Box::new(err)),
                    )
                })?;

                return Err(ClientError::new(
                    ClientErrorKind::Other("failed authenticate response"),
                    Some(errors.into()),
                ));
            },
            _ => {
                return Err(ClientError::new(
                    ClientErrorKind::Other("failed authenticate response"),
                    None,
                ));
            },
        }

        // The session JWT replaced whatever was there: reset the
        // auto-refresh schedule to this token's `exp`.
        self.arm_auto_refresh();

        Ok(true)
    }

    #[tracing::instrument(skip(self))]
    pub async fn refresh(&self) -> Result<bool, ClientError> {
        let refreshed = refresh_session(&self.config, &self.state).await?;

        self.arm_auto_refresh();

        Ok(refreshed)
    }

    /// Ends the session: stops the background auto-refresh task, then drops
    /// the JWT and refresh token from memory and, in the wasm build, from
    /// LocalStorage. Subsequent requests report `NoJwtFound`; a re-login (or
    /// `refresh` with a still-held token — gone after logout) is required.
    pub async fn logout(&self) {
        self.disarm_auto_refresh();

        let mut state = self.state.write().await;
        session::clear(&mut state);
    }

    /// Restores a raw JWT loaded from storage (wasm page reload) by verifying
    /// it against freshly fetched public keys. A token that fails verification
    /// or is already inside the refresh buffer falls through to `refresh()`;
    /// when the refresh token is spent as well, that errors and the caller
    /// logs out.
    #[tracing::instrument(level = "debug", skip(self))]
    async fn recover_jwt(&self) -> Result<bool, ClientError> {
        let raw_jwt = {
            let state = self.state.read().await;
            state.raw_jwt.clone()
        };

        let Some(raw_jwt) = raw_jwt else {
            return Ok(false);
        };

        let public_keys = self.get_public_keys().await?;

        let jwt = match verify_session_jwt(&raw_jwt, &public_keys) {
            Ok(jwt) => jwt,
            Err(_) => return self.refresh().await,
        };

        if self.needs_refresh(&jwt) {
            return self.refresh().await;
        }

        let client = bearer_client(&raw_jwt)?;

        let mut state = self.state.write().await;
        state.jwt = Some(jwt);
        state.client = client;
        session::persist(&state);

        self.arm_auto_refresh();

        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn check_auth_state(&self) -> AuthState {
        let state = self.state.read().await;

        let Some(jwt) = &state.jwt else {
            // A restored (wasm) session carries the raw JWT without its
            // decoded form: verify it against fresh public keys instead of
            // throwing the session away.
            if state.raw_jwt.is_some() {
                return AuthState::Recover;
            }

            return AuthState::Auth;
        };

        if self.needs_refresh(jwt) {
            return AuthState::Refresh;
        }

        AuthState::Valid
    }

    /// True when fewer than `refresh_buffer` seconds remain before `exp`:
    /// the token is dead, or too close to dying to ride into a request —
    /// the session rotates here instead, so the bearer on the wire always
    /// has at least the buffer left on it.
    fn needs_refresh(&self, jwt: &Jwt) -> bool {
        jwt.exp as i64 - Utc::now().timestamp()
            <= self
                .config
                .refresh_buffer
                .as_secs() as i64
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn authenticate_if_needed(&self) -> Result<bool, ClientError> {
        match self.check_auth_state().await {
            AuthState::Valid => Ok(true),
            AuthState::Auth => Ok(false),
            AuthState::Recover => self.recover_jwt().await,
            AuthState::Refresh => self.refresh().await,
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn request<Req, Res>(
        &self,
        method: Method,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.authenticate_if_needed()
            .await?;

        let state = self.state.read().await;

        let client = &state.client;

        let url = format!("{}{}", self.config.base_url, url);

        // fetch() (the wasm transport) rejects a Request with a body on
        // GET/HEAD, and the api never extracts a body on those verbs —
        // serialize the payload only where a handler can consume it.
        let request = match method {
            Method::GET | Method::HEAD => client.request(method, url),
            method => {
                client
                    .request(method, url)
                    .json(&payload)
            },
        };

        let res = request
            .send()
            .await
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::Other("http request failed"),
                    Some(Box::new(err)),
                )
            })?;

        info!(message = "oxdiauth client request response", res = ?res);

        let res = res
            .json()
            .await
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::Other("failed to deserialize response"),
                    Some(Box::new(err)),
                )
            })?;

        Ok(res)
    }

    pub async fn get<Req, Res>(&self, url: &str, payload: Req) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::GET, url, payload)
            .await
    }

    pub async fn put<Req, Res>(&self, url: &str, payload: Req) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::PUT, url, payload)
            .await
    }

    pub async fn post<Req, Res>(&self, url: &str, payload: Req) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::POST, url, payload)
            .await
    }

    pub async fn delete<Req, Res>(&self, url: &str, payload: Req) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::DELETE, url, payload)
            .await
    }

    /// (Re)arm the background auto-refresh schedule against the session JWT
    /// currently in `state`: one task wakes at `exp` minus the refresh
    /// buffer and exchanges the refresh token with no caller involvement,
    /// then reschedules itself off the rotated `exp`. Every successful
    /// session transition (`auth`, `refresh`, `recover_jwt`) re-arms, and
    /// re-arm supersedes the previous schedule — at most one is live per
    /// client, always pointed at the live token.
    #[cfg(not(target_arch = "wasm32"))]
    fn arm_auto_refresh(&self) {
        // The task holds only a *weak* handle to the state: once every
        // `Client` clone is dropped it wakes, fails to upgrade, and retires
        // rather than refreshing a session nobody owns.
        if let Some(previous) = self
            .refresh_task
            .lock()
            .take()
        {
            previous.abort();
        }

        let handle = tokio::spawn(auto_refresh_task(
            self.config.clone(),
            Arc::downgrade(&self.state),
            Cancel::AbortedExternally,
        ));

        *self.refresh_task.lock() = Some(handle.abort_handle());
    }

    /// Wasm arming is cooperative: `spawn_local` offers no abort handle, so
    /// superseding the previous schedule means bumping the shared generation
    /// — the retired loop sees the mismatch on its next wake and exits — and
    /// handing the new loop the generation it owns.
    ///
    /// The wasm schedule is best-effort by nature: hidden tabs throttle
    /// `setTimeout` (Chrome: ~1/min after five hidden minutes; a frozen tab
    /// resumes the timer late), so a wake-up can land after `exp` passed.
    /// The guarantee there stays the request-time buffer threshold in
    /// `check_auth_state`; the timer spares foreground-idle sessions from
    /// ever walking into it. An exchange already in flight cannot be
    /// cancelled by `logout` — the state write lock collapses it to a
    /// single landed pair, the same exposure a manual `refresh()` racing
    /// `logout()` has natively.
    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    fn arm_auto_refresh(&self) {
        let generation = self
            .refresh_task
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        wasm_bindgen_futures::spawn_local(auto_refresh_task(
            self.config.clone(),
            Arc::downgrade(&self.state),
            Cancel::Generation {
                slot: self.refresh_task.clone(),
                generation,
            },
        ));
    }

    /// A wasm build without the `wasm` feature has neither storage nor
    /// timer machinery: request-time refresh is the only mechanism there.
    #[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
    fn arm_auto_refresh(&self) {
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn disarm_auto_refresh(&self) {
        if let Some(handle) = self
            .refresh_task
            .lock()
            .take()
        {
            handle.abort();
        }
    }

    #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
    fn disarm_auto_refresh(&self) {
        // Any live schedule sees the generation mismatch on its next wake
        // and retires.
        self.refresh_task.fetch_add(1, Ordering::SeqCst);
    }

    #[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
    fn disarm_auto_refresh(&self) {
    }
}

#[derive(Debug)]
pub struct ClientError {
    pub kind: ClientErrorKind,
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl ClientError {
    pub fn new(
        kind: ClientErrorKind,
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> Self {
        Self { kind, source }
    }
}

#[derive(Debug)]
enum AuthState {
    Auth,
    Refresh,
    Recover,
    Valid,
}

#[derive(Debug, Copy, Clone)]
pub enum Resource {
    Auth,
    Authority,
    Permission,
    PublicKey,
    RefreshToken,
    Role,
    RolePermissionGrant,
    RoleRoleGrant,
    Setting,
    Totp,
    User,
    UserAuthority,
    UserPermissionGrant,
    UserRole,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Resource::*;

        match self {
            Auth => write!(f, "auth"),
            Authority => write!(f, "authority"),
            Permission => write!(f, "permission"),
            PublicKey => write!(f, "public_key"),
            RefreshToken => write!(f, "refresh_token"),
            Role => write!(f, "role"),
            RolePermissionGrant => write!(f, "role_permission_grant"),
            RoleRoleGrant => write!(f, "role_role_grant"),
            Setting => write!(f, "setting"),
            Totp => write!(f, "totp"),
            User => write!(f, "user"),
            UserAuthority => write!(f, "user_authority"),
            UserPermissionGrant => write!(f, "user_permission_grant"),
            UserRole => write!(f, "user_role"),
        }
    }
}

#[derive(Debug)]
pub enum ClientErrorKind {
    NoJwtFound,
    AuthError,
    RefreshError,
    EmptyPayload(Resource, &'static str),
    APIResponseError,
    UrlParseError,
    Other(&'static str),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ClientErrorKind::*;

        match self.kind {
            NoJwtFound => {
                write!(f, "no jwt found when calling get_jwt")?;
            },
            AuthError => {
                write!(f, "encountered an error authenticating")?;
            },
            RefreshError => {
                write!(f, "encountered an error while refreshing token")?;
            },
            EmptyPayload(resource, method) => {
                write!(
                    f,
                    "received an empty payload when a response payload was expcected for resource {} method {}",
                    resource, method
                )?;
            },
            APIResponseError => {
                write!(f, "error reported when making a request to the API")?;
            },
            UrlParseError => {
                write!(f, "encountered an error while parsing url")?;
            },
            Other(reason) => write!(f, "error: {}", reason)?,
        }

        let mut source = std::error::Error::source(self);
        while let Some(err) = source {
            write!(f, ": {}", err)?;
            source = err.source();
        }

        Ok(())
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            Some(err) => Some(err.as_ref()),
            None => None,
        }
    }
}

#[tracing::instrument(level = "debug")]
fn handle_response<T>(
    resource: Resource,
    method: &'static str,
    response: Response<T>,
) -> Result<T, ClientError>
where
    T: Serialize + fmt::Debug,
{
    if !response.success {
        return Err(ClientError {
            kind: ClientErrorKind::APIResponseError,
            source: response.errors.map(|err| {
                err.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .into()
            }),
        });
    }

    let payload = response
        .payload
        .ok_or_else(|| ClientError::new(ClientErrorKind::EmptyPayload(resource, method), None))?;

    Ok(payload)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::error::Error as _;

    use oxidauth_kernel::{
        base64::{BASE64_STANDARD, Engine as _},
        rsa::KeyPair,
    };
    use wiremock::{
        Mock,
        MockServer,
        ResponseTemplate,
        matchers::{body_json, method, path},
    };

    use super::*;

    const KEYS_PATH: &str = "/api/v1/public_keys";
    const AUTH_PATH: &str = "/api/v1/auth/authenticate";
    const REFRESH_PATH: &str = "/api/v1/refresh_tokens";

    fn keypair() -> KeyPair {
        KeyPair::new().unwrap()
    }

    fn public_key_entry(public_key: &str) -> serde_json::Value {
        json!({
            "id": Uuid::new_v4().to_string(),
            "public_key": public_key,
            "created_at": "2026-09-29T00:00:00Z",
            "updated_at": "2026-09-29T00:00:00Z",
        })
    }

    // Happy-path keyset (test scaffolding). What the API *serves* on the wire is
    // RAW PEM: the DB stores base64(PEM) (`create_public_key`), but the
    // ListAllPublicKeysUseCase (services/public_keys/list_all_public_keys.rs)
    // base64-DECODES every key back to raw PEM before returning it. This fixture
    // therefore mounts exactly one raw-PEM entry — the real wire format — so
    // every happy-path state-machine test drives refresh() against it
    // (OXA-000029). The base64(PEM) storage encoding is exercised only by the
    // dedicated version-skew fallback / rejection tests below.
    fn full_keyset(kp: &KeyPair) -> Vec<serde_json::Value> {
        vec![public_key_entry(std::str::from_utf8(&kp.public).unwrap())]
    }

    // Real RS256 tokens so expiry handling (leeway + `check_auth_state`) is genuine.
    fn mint(private_pem: &[u8], sub: Uuid, exp_offset: i64) -> (String, usize) {
        let exp = (Utc::now().timestamp() + exp_offset) as usize;

        let raw = Jwt::builder()
            .with_subject(sub)
            .with_expires_at(exp)
            .build()
            .unwrap()
            .encode(private_pem)
            .unwrap();

        (raw, exp)
    }

    fn envelope(payload: serde_json::Value) -> serde_json::Value {
        json!({ "success": true, "payload": payload })
    }

    fn auth_payload(jwt: &str, refresh_token: Uuid) -> serde_json::Value {
        json!({ "jwt": jwt, "refresh_token": refresh_token.to_string() })
    }

    // ExchangeRefreshTokenRes == AuthenticateResponse: jwt + refresh_token + user_id.
    fn refresh_payload(jwt: &str, refresh_token: Uuid) -> serde_json::Value {
        json!({
            "jwt": jwt,
            "refresh_token": refresh_token.to_string(),
            "user_id": Uuid::new_v4().to_string(),
        })
    }

    async fn mount_json(
        server: &MockServer,
        m: &str,
        p: &str,
        status: u16,
        resp: serde_json::Value,
    ) {
        Mock::given(method(m))
            .and(path(p))
            .respond_with(ResponseTemplate::new(status).set_body_json(resp))
            .mount(server)
            .await;
    }

    async fn mount_keys(server: &MockServer, entries: Vec<serde_json::Value>) {
        mount_json(
            server,
            "GET",
            KEYS_PATH,
            200,
            envelope(json!({ "public_keys": entries })),
        )
        .await;
    }

    async fn client_for(server: &MockServer) -> (Client, Uuid) {
        let client_key = Uuid::new_v4();
        let base_url = Url::parse(&server.uri()).unwrap();

        (Client::new(&base_url, client_key).unwrap(), client_key)
    }

    async fn received(server: &MockServer, m: &str, p: &str) -> Vec<wiremock::Request> {
        server
            .received_requests()
            .await
            .unwrap()
            .into_iter()
            .filter(|req| req.method.as_str() == m && req.url.path() == p)
            .collect()
    }

    fn body(req: &wiremock::Request) -> serde_json::Value {
        serde_json::from_slice(&req.body).unwrap()
    }

    async fn authenticate_ok(server: &MockServer, kp: &KeyPair, jwt: &str, rt: Uuid) {
        mount_keys(server, full_keyset(kp)).await;
        mount_json(
            server,
            "POST",
            AUTH_PATH,
            200,
            envelope(auth_payload(jwt, rt)),
        )
        .await;
    }

    // ---------------------------------------------------------------- E1

    #[tokio::test]
    async fn authenticate_success_stores_jwt_and_caches_it() {
        let server = MockServer::start().await;
        let kp = keypair();

        let sub = Uuid::new_v4();
        let refresh_token = Uuid::new_v4();
        let (jwt, _exp) = mint(&kp.private, sub, 3600);
        authenticate_ok(&server, &kp, &jwt, refresh_token).await;

        let (client, client_key) = client_for(&server).await;

        assert!(
            client
                .authenticate("malreynolds", "password123")
                .await
                .unwrap()
        );

        // request contract: route, verb and body the client sends
        let auth_calls = received(&server, "POST", AUTH_PATH).await;
        assert_eq!(auth_calls.len(), 1);
        let req_body = body(&auth_calls[0]);
        assert_eq!(req_body["client_key"], client_key.to_string());
        assert_eq!(req_body["params"]["username"], "malreynolds");
        assert_eq!(req_body["params"]["password"], "password123");

        // stored state
        assert_eq!(
            client
                .get_jwt()
                .await
                .unwrap(),
            jwt
        );
        assert_eq!(
            client
                .get_jwt_decoded()
                .await
                .unwrap()
                .sub,
            Some(sub)
        );

        // caching pin: the token is unexpired, so further calls must not
        // refetch keys or re-authenticate
        client
            .get_jwt()
            .await
            .unwrap();
        client
            .get_jwt_decoded()
            .await
            .unwrap();
        assert_eq!(
            received(&server, "GET", KEYS_PATH)
                .await
                .len(),
            1
        );
        assert_eq!(
            received(&server, "POST", AUTH_PATH)
                .await
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn get_jwt_without_authentication_never_hits_the_network() {
        let server = MockServer::start().await;
        let (client, _) = client_for(&server).await;

        // check_auth_state -> AuthState::Auth: authenticate_if_needed returns
        // Ok(false) (the client cannot self-authenticate, it stores no
        // credentials) and get_jwt surfaces NoJwtFound — zero HTTP.
        let err = client
            .get_jwt()
            .await
            .unwrap_err();
        assert!(matches!(err.kind, ClientErrorKind::NoJwtFound));
        assert_eq!(format!("{err}"), "no jwt found when calling get_jwt");
        assert!(err.source().is_none());
        assert!(
            server
                .received_requests()
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn authenticate_failure_envelope_surfaces_server_errors_in_source() {
        let server = MockServer::start().await;
        let kp = keypair();
        mount_keys(&server, full_keyset(&kp)).await;
        mount_json(
            &server,
            "POST",
            AUTH_PATH,
            401,
            json!({ "success": false, "errors": ["invalid credentials"] }),
        )
        .await;

        let (client, _) = client_for(&server).await;
        let err = client
            .authenticate("u", "bad")
            .await
            .unwrap_err();

        // the client keys off the envelope; the 401 status itself is never consulted
        assert!(matches!(
            err.kind,
            ClientErrorKind::Other("failed authenticate response")
        ));
        assert_eq!(
            err.source()
                .unwrap()
                .to_string(),
            r#"["invalid credentials"]"#,
            "success:false errors are surfaced as a serialized JSON string"
        );

        // state untouched on failure
        assert!(matches!(
            client
                .get_jwt()
                .await
                .unwrap_err()
                .kind,
            ClientErrorKind::NoJwtFound
        ));
    }

    #[tokio::test]
    async fn authenticate_rejects_jwt_signed_with_foreign_key() {
        let server = MockServer::start().await;
        let server_kp = keypair();
        let attacker_kp = keypair();

        let sub = Uuid::new_v4();
        let (foreign_jwt, _) = mint(&attacker_kp.private, sub, 3600);
        authenticate_ok(&server, &server_kp, &foreign_jwt, Uuid::new_v4()).await;

        let (client, _) = client_for(&server).await;
        let err = client
            .authenticate("u", "p")
            .await
            .unwrap_err();

        assert!(matches!(
            err.kind,
            ClientErrorKind::Other("failed to validate jwt")
        ));
        assert!(err.source().is_none());
        assert!(matches!(
            client
                .get_jwt()
                .await
                .unwrap_err()
                .kind,
            ClientErrorKind::NoJwtFound
        ));
    }

    #[tokio::test]
    async fn expired_jwt_triggers_refresh_via_get_jwt() {
        let server = MockServer::start().await;
        let kp = keypair();

        // exp 30s in the past: Jwt::decode still accepts it (jsonwebtoken default
        // 60s leeway) but check_auth_state (now > exp) classifies it Refresh.
        let sub = Uuid::new_v4();
        let (stale_jwt, _) = mint(&kp.private, sub, -30);
        let stale_rt = Uuid::new_v4();
        authenticate_ok(&server, &kp, &stale_jwt, stale_rt).await;

        let (fresh_jwt, fresh_exp) = mint(&kp.private, sub, 3600);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&fresh_jwt, Uuid::new_v4())),
        )
        .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        // get_jwt -> authenticate_if_needed -> AuthState::Refresh -> refresh()
        let returned = client
            .get_jwt()
            .await
            .unwrap();

        assert_eq!(
            received(&server, "GET", KEYS_PATH)
                .await
                .len(),
            2
        );
        let refresh_calls = received(&server, "POST", REFRESH_PATH).await;
        assert_eq!(refresh_calls.len(), 1);
        assert_eq!(
            body(&refresh_calls[0])["refresh_token"],
            stale_rt.to_string(),
            "refresh posts the stored refresh_token"
        );

        assert_eq!(returned, fresh_jwt);

        // the decoded side *does* see the refreshed token
        assert_eq!(
            client
                .get_jwt_decoded()
                .await
                .unwrap()
                .exp,
            fresh_exp
        );

        // refreshed token is Valid: no further HTTP
        assert_eq!(
            server
                .received_requests()
                .await
                .unwrap()
                .len(),
            4
        );
    }

    // CLI-2 regression: after an expiry-driven refresh, the raw accessor and
    // the bearer actually sent on the wire must agree on the fresh token.
    #[tokio::test]
    async fn get_jwt_after_refresh_matches_the_bearer_on_the_wire() {
        let server = MockServer::start().await;
        let kp = keypair();

        // exp 30s in the past: decode still accepts it (leeway) but
        // check_auth_state classifies it Refresh.
        let sub = Uuid::new_v4();
        let (stale_jwt, _) = mint(&kp.private, sub, -30);
        authenticate_ok(&server, &kp, &stale_jwt, Uuid::new_v4()).await;

        let (fresh_jwt, _) = mint(&kp.private, sub, 3600);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&fresh_jwt, Uuid::new_v4())),
        )
        .await;
        mount_json(
            &server,
            "GET",
            "/api/v1/users",
            200,
            envelope(json!({ "ok": true })),
        )
        .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        // get_jwt -> authenticate_if_needed -> AuthState::Refresh -> refresh()
        let returned = client
            .get_jwt()
            .await
            .unwrap();
        assert_eq!(returned, fresh_jwt, "get_jwt must return the fresh token");

        // a subsequent request() rides the same refreshed credential
        let res: Response<serde_json::Value> = client
            .get("/users", None::<()>)
            .await
            .unwrap();
        assert!(res.success);

        let jwt = client
            .get_jwt()
            .await
            .unwrap();
        assert_eq!(jwt, fresh_jwt);
        let user_calls = received(&server, "GET", "/api/v1/users").await;
        assert_eq!(user_calls.len(), 1);
        assert_eq!(
            user_calls[0]
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok()),
            Some(format!("Bearer {jwt}").as_str()),
            "the wire bearer must equal Bearer <get_jwt()>"
        );
    }

    #[tokio::test]
    async fn refresh_rotation_persists_rotated_token_for_next_refresh() {
        let server = MockServer::start().await;
        let kp = keypair();

        let sub = Uuid::new_v4();
        let (jwt, _) = mint(&kp.private, sub, 3600);
        let rt1 = Uuid::new_v4();
        authenticate_ok(&server, &kp, &jwt, rt1).await;

        // route the two refresh responses by the token the client sends
        let rt2 = Uuid::new_v4();
        let rt3 = Uuid::new_v4();
        let (jwt2, _) = mint(&kp.private, sub, 7200);
        let (jwt3, _) = mint(&kp.private, sub, 7200);

        Mock::given(method("POST"))
            .and(path(REFRESH_PATH))
            .and(body_json(json!({ "refresh_token": rt1.to_string() })))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(envelope(refresh_payload(&jwt2, rt2))),
            )
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(REFRESH_PATH))
            .and(body_json(json!({ "refresh_token": rt2.to_string() })))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(envelope(refresh_payload(&jwt3, rt3))),
            )
            .mount(&server)
            .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        assert!(
            client
                .refresh()
                .await
                .unwrap()
        );
        assert!(
            client
                .refresh()
                .await
                .unwrap()
        );

        let bodies: Vec<serde_json::Value> = received(&server, "POST", REFRESH_PATH)
            .await
            .iter()
            .map(body)
            .collect();
        assert_eq!(
            bodies,
            vec![
                json!({ "refresh_token": rt1.to_string() }),
                json!({ "refresh_token": rt2.to_string() }),
            ],
            "the rotated refresh_token from response #1 must be spent on refresh #2"
        );

        // every refresh refetches the keyset (auth + 2 refreshes)
        assert_eq!(
            received(&server, "GET", KEYS_PATH)
                .await
                .len(),
            3
        );
    }

    #[tokio::test]
    async fn refresh_without_stored_token_fails_after_refetching_keys() {
        let server = MockServer::start().await;
        let kp = keypair();
        mount_keys(&server, full_keyset(&kp)).await;

        let (client, _) = client_for(&server).await;
        let err = client
            .refresh()
            .await
            .unwrap_err();

        assert!(matches!(
            err.kind,
            ClientErrorKind::Other("can't refresh -- no refresh token found")
        ));
        // pin ordering: keys are fetched *before* the refresh-token check
        assert_eq!(
            received(&server, "GET", KEYS_PATH)
                .await
                .len(),
            1
        );
        assert!(
            received(&server, "POST", REFRESH_PATH)
                .await
                .is_empty()
        );
    }

    #[tokio::test]
    async fn refresh_failure_envelope_collapses_to_an_empty_error() {
        let server = MockServer::start().await;
        let kp = keypair();
        let sub = Uuid::new_v4();
        let (jwt, exp) = mint(&kp.private, sub, 3600);
        authenticate_ok(&server, &kp, &jwt, Uuid::new_v4()).await;
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            400,
            json!({ "success": false, "errors": ["refresh token has expired"] }),
        )
        .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        let err = client
            .refresh()
            .await
            .unwrap_err();

        // BUG(pinned): the failure arm is `Other("")` — the server-reported
        // errors are dropped and the error renders as the bare string "error: ".
        assert!(matches!(err.kind, ClientErrorKind::Other("")));
        assert_eq!(format!("{err}"), "error: ");
        assert!(err.source().is_none());

        // previous auth state survives the failed refresh
        assert_eq!(
            client
                .get_jwt()
                .await
                .unwrap(),
            jwt
        );
        assert_eq!(
            client
                .get_jwt_decoded()
                .await
                .unwrap()
                .exp,
            exp
        );
    }

    #[tokio::test]
    async fn authenticated_request_carries_bearer_token_without_reauth() {
        let server = MockServer::start().await;
        let kp = keypair();
        let sub = Uuid::new_v4();
        let (jwt, _) = mint(&kp.private, sub, 3600);
        authenticate_ok(&server, &kp, &jwt, Uuid::new_v4()).await;
        mount_json(
            &server,
            "GET",
            "/api/v1/users",
            200,
            envelope(json!({ "ok": true })),
        )
        .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        let res: Response<serde_json::Value> = client
            .get("/users", None::<()>)
            .await
            .unwrap();
        assert!(res.success);
        assert_eq!(res.payload.unwrap()["ok"], true);

        let user_calls = received(&server, "GET", "/api/v1/users").await;
        assert_eq!(user_calls.len(), 1);
        assert_eq!(
            user_calls[0]
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok()),
            Some(format!("Bearer {jwt}").as_str()),
            "auth() must install the bearer on the state client"
        );

        // Valid state: the endpoint call did not re-run keys/authenticate
        assert_eq!(
            received(&server, "GET", KEYS_PATH)
                .await
                .len(),
            1
        );
        assert_eq!(
            received(&server, "POST", AUTH_PATH)
                .await
                .len(),
            1
        );
    }

    // ---------------------------------------------------------------- E2

    #[tokio::test]
    async fn public_keys_transport_failure_maps_to_fetch_error() {
        // nothing listens on 127.0.0.1:1 -> reqwest send error, no wiremock needed
        let base_url = Url::parse("http://127.0.0.1:1").unwrap();
        let client = Client::new(&base_url, Uuid::new_v4()).unwrap();

        let err = client
            .authenticate("u", "p")
            .await
            .unwrap_err();
        assert!(matches!(
            err.kind,
            ClientErrorKind::Other("unable to fetch public keys")
        ));
        assert!(err.source().is_some(), "reqwest error must be chained");
    }

    #[tokio::test]
    async fn public_keys_non_json_body_maps_to_deserialize_error() {
        // 200 + non-JSON body
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(KEYS_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>"))
            .mount(&server)
            .await;
        let (client, _) = client_for(&server).await;
        let err = client
            .authenticate("u", "p")
            .await
            .unwrap_err();
        assert!(matches!(
            err.kind,
            ClientErrorKind::Other("unable to deserialize public keys")
        ));

        // 404 + empty body: status is ignored, the .json() failure is what surfaces
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(KEYS_PATH))
            .respond_with(ResponseTemplate::new(404).set_body_string(""))
            .mount(&server)
            .await;
        let (client, _) = client_for(&server).await;
        assert!(matches!(
            client
                .authenticate("u", "p")
                .await
                .unwrap_err()
                .kind,
            ClientErrorKind::Other("unable to deserialize public keys")
        ));
    }

    #[tokio::test]
    async fn empty_keyset_is_rejected_before_authenticate() {
        let server = MockServer::start().await;
        mount_keys(&server, vec![]).await;

        let (client, _) = client_for(&server).await;
        let err = client
            .authenticate("u", "p")
            .await
            .unwrap_err();

        assert!(matches!(
            err.kind,
            ClientErrorKind::Other("no public keys found")
        ));
        assert_eq!(
            received(&server, "GET", KEYS_PATH)
                .await
                .len(),
            1
        );
        assert!(
            received(&server, "POST", AUTH_PATH)
                .await
                .is_empty(),
            "auth must short-circuit on an empty keyset"
        );
    }

    #[tokio::test]
    async fn unsuccessful_or_payloadless_keys_envelope_maps_to_deserialize_error() {
        // success:false envelope
        let server = MockServer::start().await;
        mount_json(
            &server,
            "GET",
            KEYS_PATH,
            200,
            json!({ "success": false, "errors": ["store unavailable"] }),
        )
        .await;
        let (client, _) = client_for(&server).await;
        // pin actual copy: a reported failure is mislabeled as a *deserialize* problem
        assert!(matches!(
            client
                .authenticate("u", "p")
                .await
                .unwrap_err()
                .kind,
            ClientErrorKind::Other("failed to deserialize public keys")
        ));

        // success:true without payload
        let server = MockServer::start().await;
        mount_json(&server, "GET", KEYS_PATH, 200, json!({ "success": true })).await;
        let (client, _) = client_for(&server).await;
        assert!(matches!(
            client
                .authenticate("u", "p")
                .await
                .unwrap_err()
                .kind,
            ClientErrorKind::Other("failed to deserialize public keys")
        ));
    }

    #[tokio::test]
    async fn auth_rejects_base64_storage_format_the_api_never_serves() {
        let server = MockServer::start().await;
        let kp = keypair();
        // the *DB storage* format (create_public_key base64-encodes); production
        // never sees it on the wire — the /public_keys endpoint decodes to raw PEM
        // (ListAllPublicKeysUseCase) before serving.
        mount_keys(
            &server,
            vec![public_key_entry(&BASE64_STANDARD.encode(&kp.public))],
        )
        .await;

        let sub = Uuid::new_v4();
        let (jwt, _) = mint(&kp.private, sub, 3600);
        mount_json(
            &server,
            "POST",
            AUTH_PATH,
            200,
            envelope(auth_payload(&jwt, Uuid::new_v4())),
        )
        .await;

        let (client, _) = client_for(&server).await;
        let err = client
            .authenticate("u", "p")
            .await
            .unwrap_err();

        // Pin (not a bug): auth()'s decode_with_public_keys feeds the wire string
        // verbatim into from_rsa_pem — consistent with the raw-PEM wire format the
        // API actually serves, i.e. the healthy production path. Do NOT "fix"
        // auth() to base64-decode: that would sink the format it really receives.
        // The format bug belongs to refresh() only.
        assert!(matches!(
            err.kind,
            ClientErrorKind::Other("failed to validate jwt")
        ));
    }

    #[tokio::test]
    async fn refresh_validates_the_real_raw_pem_wire_format() {
        let server = MockServer::start().await;
        let kp = keypair();
        // raw PEM = the exact format /public_keys serves in production (the use
        // case base64-decodes DB values before returning). auth() verifies fine
        // with it, and so must refresh() (OXA-000029).
        let sub = Uuid::new_v4();
        let (jwt, _) = mint(&kp.private, sub, 3600);
        mount_keys(
            &server,
            vec![public_key_entry(std::str::from_utf8(&kp.public).unwrap())],
        )
        .await;
        mount_json(
            &server,
            "POST",
            AUTH_PATH,
            200,
            envelope(auth_payload(&jwt, Uuid::new_v4())),
        )
        .await;
        // the refresh response itself is fine — only the keyset format can sink it
        let (refreshed, fresh_exp) = mint(&kp.private, sub, 7200);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&refreshed, Uuid::new_v4())),
        )
        .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        // refresh() verifies the raw-PEM key it was served (the raw leg of
        // `Jwt::decode_with_flexible_public_keys`) instead of demanding the
        // storage encoding it used to assume.
        assert!(
            client
                .refresh()
                .await
                .unwrap()
        );
        // get_jwt() equality after refresh is covered by the CLI-2/OXA-000028
        // regression test (get_jwt_after_refresh_matches_the_bearer_on_the_wire).
        assert_eq!(
            client
                .get_jwt_decoded()
                .await
                .unwrap()
                .exp,
            fresh_exp
        );
    }

    // Version-skew insurance (OXA-000029): if an endpoint ever serves keys in
    // the storage / create / find-by-id encoding, base64(PEM), refresh()'s
    // fallback leg still validates the rotated jwt. auth() cannot get past a
    // b64-only keyset (pinned by the test above), so the session is seeded
    // directly and only refresh() is driven.
    #[tokio::test]
    async fn refresh_falls_back_to_the_base64_storage_format() {
        let server = MockServer::start().await;
        let kp = keypair();
        mount_keys(
            &server,
            vec![public_key_entry(&BASE64_STANDARD.encode(&kp.public))],
        )
        .await;

        let sub = Uuid::new_v4();
        let (refreshed, fresh_exp) = mint(&kp.private, sub, 7200);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&refreshed, Uuid::new_v4())),
        )
        .await;

        let (client, _) = client_for(&server).await;
        client
            .state
            .write()
            .await
            .refresh_token = Some(Uuid::new_v4());

        assert!(
            client
                .refresh()
                .await
                .unwrap()
        );
        assert_eq!(
            client
                .get_jwt_decoded()
                .await
                .unwrap()
                .exp,
            fresh_exp
        );
        assert_eq!(
            received(&server, "POST", REFRESH_PATH)
                .await
                .len(),
            1
        );
    }

    // The fallback leg must not become rubber-stamping (OXA-000029): keys that
    // cannot validate the rotated jwt are skipped in EITHER encoding — foreign
    // raw PEM, foreign base64(PEM), and base64-of-non-matching-material alike
    // — and the error keeps its shape.
    #[tokio::test]
    async fn refresh_rejects_foreign_keys_in_either_encoding() {
        let server = MockServer::start().await;
        let server_kp = keypair();
        let foreign_kp = keypair();
        let rogue_kp = keypair();

        let sub = Uuid::new_v4();
        let (jwt, exp) = mint(&server_kp.private, sub, 3600);
        // the keyset carries the server's raw PEM (auth's healthy path) plus a
        // foreign key in BOTH encodings; the rotated jwt is signed by a third
        // key that appears nowhere on the wire
        mount_keys(
            &server,
            vec![
                public_key_entry(std::str::from_utf8(&server_kp.public).unwrap()),
                public_key_entry(std::str::from_utf8(&foreign_kp.public).unwrap()),
                public_key_entry(&BASE64_STANDARD.encode(&foreign_kp.public)),
            ],
        )
        .await;
        mount_json(
            &server,
            "POST",
            AUTH_PATH,
            200,
            envelope(auth_payload(&jwt, Uuid::new_v4())),
        )
        .await;
        let (rogue_refreshed, _) = mint(&rogue_kp.private, sub, 7200);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&rogue_refreshed, Uuid::new_v4())),
        )
        .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        let err = client
            .refresh()
            .await
            .unwrap_err();
        assert!(matches!(
            err.kind,
            ClientErrorKind::Other("failed to validate jwt")
        ));

        // the failed refresh leaves the prior session intact
        assert_eq!(
            client
                .get_jwt_decoded()
                .await
                .unwrap()
                .exp,
            exp
        );
    }

    // E1 concurrency clause: N simultaneous get_jwt() calls on an expired token.
    // check_auth_state re-acquires the *read* lock after the first refresh
    // releases the write lock refresh() holds across its whole keys+refresh
    // exchange, so every queued caller re-reads fresh state before deciding —
    // asserting below that exactly ONE refresh POST happens (no thundering herd).
    #[tokio::test]
    async fn concurrent_get_jwt_after_expiry_refreshes_exactly_once() {
        let server = MockServer::start().await;
        let kp = keypair();

        let sub = Uuid::new_v4();
        let (stale_jwt, _) = mint(&kp.private, sub, -30);
        authenticate_ok(&server, &kp, &stale_jwt, Uuid::new_v4()).await;

        let (fresh_jwt, _) = mint(&kp.private, sub, 3600);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&fresh_jwt, Uuid::new_v4())),
        )
        .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        let (r1, r2, r3, r4, r5, r6, r7, r8) = tokio::join!(
            client.get_jwt(),
            client.get_jwt(),
            client.get_jwt(),
            client.get_jwt(),
            client.get_jwt(),
            client.get_jwt(),
            client.get_jwt(),
            client.get_jwt(),
        );

        for r in [r1, r2, r3, r4, r5, r6, r7, r8] {
            // refresh() writes raw_jwt in its validated arm, so every caller
            // sees the same refreshed token string.
            assert_eq!(r.unwrap(), fresh_jwt);
        }

        assert_eq!(
            received(&server, "POST", REFRESH_PATH)
                .await
                .len(),
            1,
            "one refresh must serve every waiting caller"
        );
        // auth keys-fetch + refresh keys-fetch, nothing more
        assert_eq!(
            received(&server, "GET", KEYS_PATH)
                .await
                .len(),
            2
        );
    }

    // ---------------------------------------------------------------- E3

    #[test]
    fn client_error_display_is_pinned() {
        use ClientErrorKind::*;

        let cases: Vec<(ClientErrorKind, &str)> = vec![
            (NoJwtFound, "no jwt found when calling get_jwt"),
            (AuthError, "encountered an error authenticating"),
            (RefreshError, "encountered an error while refreshing token"),
            (
                // pin actual copy incl. the "expcected" typo
                EmptyPayload(Resource::UserAuthority, "create_user_authority"),
                "received an empty payload when a response payload was expcected \
                 for resource user_authority method create_user_authority",
            ),
            (
                APIResponseError,
                "error reported when making a request to the API",
            ),
            (UrlParseError, "encountered an error while parsing url"),
            (Other("boom"), "error: boom"),
        ];
        for (kind, want) in cases {
            assert_eq!(ClientError::new(kind, None).to_string(), want);
        }

        let resources: Vec<(Resource, &str)> = vec![
            (Resource::Auth, "auth"),
            (Resource::Authority, "authority"),
            (Resource::Permission, "permission"),
            (Resource::PublicKey, "public_key"),
            (Resource::RefreshToken, "refresh_token"),
            (Resource::Role, "role"),
            (Resource::RolePermissionGrant, "role_permission_grant"),
            (Resource::RoleRoleGrant, "role_role_grant"),
            (Resource::Setting, "setting"),
            (Resource::Totp, "totp"),
            (Resource::User, "user"),
            (Resource::UserAuthority, "user_authority"),
            (Resource::UserPermissionGrant, "user_permission_grant"),
            (Resource::UserRole, "user_role"),
        ];
        for (resource, want) in resources {
            assert_eq!(resource.to_string(), want);
        }
    }

    #[test]
    fn client_error_source_maps_inner_error_or_nothing() {
        let err = ClientError::new(ClientErrorKind::AuthError, Some(Box::from("inner reason")));
        assert_eq!(
            err.source()
                .unwrap()
                .to_string(),
            "inner reason"
        );

        // reachable through the std Error trait as well
        let as_dyn: &dyn std::error::Error = &err;
        assert_eq!(
            as_dyn
                .source()
                .unwrap()
                .to_string(),
            "inner reason"
        );

        let err = ClientError::new(ClientErrorKind::AuthError, None);
        assert!(err.source().is_none());
    }

    #[test]
    fn handle_response_returns_payload_on_success() {
        let res = handle_response(
            Resource::User,
            "get_user",
            Response::<serde_json::Value>::success().payload(json!({ "id": 7 })),
        )
        .unwrap();

        assert_eq!(res, json!({ "id": 7 }));
    }

    #[test]
    fn handle_response_api_error_joins_errors_into_source() {
        let err = handle_response(
            Resource::Role,
            "get_role",
            Response::<serde_json::Value>::bad_request()
                .error("e1")
                .error("e2"),
        )
        .unwrap_err();

        assert!(matches!(err.kind, ClientErrorKind::APIResponseError));
        assert_eq!(
            err.source()
                .unwrap()
                .to_string(),
            "\"e1\", \"e2\"",
            "errors serialize to JSON values, so the joined string carries quotes"
        );

        // success:false without errors -> no source
        let err = handle_response(
            Resource::Role,
            "get_role",
            Response::<serde_json::Value>::internal_error(),
        )
        .unwrap_err();
        assert!(matches!(err.kind, ClientErrorKind::APIResponseError));
        assert!(err.source().is_none());
    }

    #[test]
    fn handle_response_empty_payload_names_resource_and_method() {
        let err = handle_response(
            Resource::User,
            "get_user",
            Response::<serde_json::Value>::success(),
        )
        .unwrap_err();

        assert!(matches!(
            err.kind,
            ClientErrorKind::EmptyPayload(Resource::User, "get_user")
        ));
        assert_eq!(
            err.to_string(),
            "received an empty payload when a response payload was expcected \
             for resource user method get_user"
        );
        assert!(err.source().is_none());
    }

    #[test]
    fn client_new_rejects_base_that_cannot_host_a_path() {
        // cannot-be-a-base URL: join("/api/v1") fails -> UrlParseError w/ source
        let err = Client::new(
            &Url::parse("mailto:foo@example.com").unwrap(),
            Uuid::new_v4(),
        )
        .unwrap_err();

        assert!(matches!(err.kind, ClientErrorKind::UrlParseError));
        assert!(
            err.to_string()
                .starts_with("encountered an error while parsing url")
        );
        // the Display impl appends the full source chain.
        assert!(err.to_string().len() > "encountered an error while parsing url".len());
        assert!(err.source().is_some());
    }

    #[test]
    fn client_new_joins_api_version_prefix_onto_base_url() {
        let client_key = Uuid::new_v4();
        // an absolute join replaces any path on the base
        let base = Url::parse("http://oxidauth.test/some/deep/path").unwrap();
        let client = Client::new(&base, client_key).unwrap();

        assert_eq!(
            client
                .config
                .base_url
                .as_str(),
            "http://oxidauth.test/api/v1"
        );
        assert_eq!(client.config.client_key, client_key);
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_client_is_preflighted_with_mock_jwt_but_empty_state() {
        let jwt = Jwt::builder()
            .with_subject(Uuid::new_v4())
            .with_expires_at((Utc::now().timestamp() + 3600) as usize)
            .build()
            .unwrap();

        let client = Client::test_client(jwt).unwrap();

        assert!(client.mock_jwt.is_some());
        assert_eq!(
            client
                .config
                .base_url
                .as_str(),
            "http://base_url.com/api/v1",
            "test_client hard-codes its base_url + random client_key"
        );

        // the mock jwt is *not* injected into the auth state
        assert!(matches!(
            client
                .get_jwt()
                .await
                .unwrap_err()
                .kind,
            ClientErrorKind::NoJwtFound
        ));
    }

    // P1 session contract, native half: `logout` must wipe both the JWT and
    // the refresh token from the in-memory state. The wasm half — the same
    // wipe reaching LocalStorage — shares `session::clear` and is pinned by
    // the `crate::wasm` storage tests (js-sys panics off-wasm).
    #[tokio::test]
    async fn logout_drops_session_and_refresh_token() {
        let server = MockServer::start().await;
        let kp = keypair();

        let (jwt, _exp) = mint(&kp.private, Uuid::new_v4(), 3600);
        authenticate_ok(&server, &kp, &jwt, Uuid::new_v4()).await;

        let (client, _client_key) = client_for(&server).await;
        client
            .authenticate("malreynolds", "password123")
            .await
            .unwrap();

        client.logout().await;

        assert!(matches!(
            client
                .get_jwt()
                .await
                .unwrap_err()
                .kind,
            ClientErrorKind::NoJwtFound
        ));

        assert!(matches!(
            client
                .refresh()
                .await
                .unwrap_err()
                .kind,
            ClientErrorKind::Other(reason) if reason == "can't refresh -- no refresh token found"
        ));
    }

    // ---------------------------------------------------------------- E4
    // Proactive JWT refresh: the background task rotates the session at
    // `exp - refresh_buffer` with zero caller involvement (native builds);
    // `check_auth_state` keeps the same window as a request-time threshold.

    async fn wait_for_refresh(server: &MockServer) -> Vec<wiremock::Request> {
        let mut calls = vec![];
        for _ in 0..200 {
            calls = received(server, "POST", REFRESH_PATH).await;
            if !calls.is_empty() {
                return calls;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        calls
    }

    #[test]
    fn refresh_buffer_defaults_to_15s_until_overridden() {
        let base = Url::parse("http://oxidauth.test").unwrap();
        let client = Client::new(&base, Uuid::new_v4()).unwrap();

        assert_eq!(DEFAULT_JWT_REFRESH_BUFFER, Duration::from_secs(15));
        assert_eq!(client.config.refresh_buffer, Duration::from_secs(15));

        let client = client.with_refresh_buffer(Duration::from_millis(2_500));
        assert_eq!(client.config.refresh_buffer, Duration::from_millis(2_500));
    }

    #[tokio::test]
    async fn auto_refresh_rotates_the_jwt_before_expiry_without_any_call() {
        let server = MockServer::start().await;
        let kp = keypair();

        let sub = Uuid::new_v4();
        let rt = Uuid::new_v4();
        // 2s of life, 1s buffer: the task must fire ~1s after login
        let (jwt, _) = mint(&kp.private, sub, 2);
        authenticate_ok(&server, &kp, &jwt, rt).await;

        let (fresh_jwt, fresh_exp) = mint(&kp.private, sub, 7200);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&fresh_jwt, Uuid::new_v4())),
        )
        .await;

        let base_url = Url::parse(&server.uri()).unwrap();
        let client = Client::new(&base_url, Uuid::new_v4())
            .unwrap()
            .with_refresh_buffer(Duration::from_secs(1));

        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        // nothing else touches the client: only the background task can
        // produce the exchange
        let refresh_calls = wait_for_refresh(&server).await;
        assert_eq!(
            refresh_calls.len(),
            1,
            "the task must exchange before expiry, with no caller asking"
        );
        assert_eq!(
            body(&refresh_calls[0])["refresh_token"],
            rt.to_string(),
            "the exchange spends the login's refresh_token"
        );

        // the rotation reaches both accessors, and the re-armed schedule for
        // the 7200s token stays silent
        assert_eq!(
            client
                .get_jwt()
                .await
                .unwrap(),
            fresh_jwt
        );
        assert_eq!(
            client
                .get_jwt_decoded()
                .await
                .unwrap()
                .exp,
            fresh_exp
        );
        assert_eq!(
            received(&server, "POST", REFRESH_PATH)
                .await
                .len(),
            1,
            "the fresh schedule is an hour out: zero further traffic"
        );
    }

    #[tokio::test]
    async fn request_inside_the_refresh_buffer_rotates_before_it_is_sent() {
        let server = MockServer::start().await;
        let kp = keypair();

        let sub = Uuid::new_v4();
        // 10s of life < the default 15s buffer: the session counts as spent
        // although it is not expired
        let (jwt, _) = mint(&kp.private, sub, 10);
        authenticate_ok(&server, &kp, &jwt, Uuid::new_v4()).await;

        let (fresh_jwt, _) = mint(&kp.private, sub, 7200);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&fresh_jwt, Uuid::new_v4())),
        )
        .await;
        mount_json(
            &server,
            "GET",
            "/api/v1/users",
            200,
            envelope(json!({ "ok": true })),
        )
        .await;

        let (client, _) = client_for(&server).await;
        assert!(
            client
                .authenticate("u", "p")
                .await
                .unwrap()
        );

        // login armed the task on a token already inside the buffer: it must
        // retire without exchanging — a sub-buffer authority TTL would
        // otherwise hammer the api
        tokio::time::sleep(Duration::from_millis(250)).await;
        assert!(
            received(&server, "POST", REFRESH_PATH)
                .await
                .is_empty(),
            "a non-positive lead must not exchange in a loop"
        );

        let res: Response<serde_json::Value> = client
            .get("/users", None::<()>)
            .await
            .unwrap();
        assert!(res.success);

        assert_eq!(
            received(&server, "POST", REFRESH_PATH)
                .await
                .len(),
            1,
            "the request path rotates the in-buffer token before sending"
        );
        let user_calls = received(&server, "GET", "/api/v1/users").await;
        assert_eq!(user_calls.len(), 1);
        assert_eq!(
            user_calls[0]
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok()),
            Some(format!("Bearer {fresh_jwt}").as_str()),
            "the wire bearer is the rotated token, never the near-dead one"
        );
    }

    #[tokio::test]
    async fn logout_disarms_the_auto_refresh_task() {
        let server = MockServer::start().await;
        let kp = keypair();

        let (jwt, _) = mint(&kp.private, Uuid::new_v4(), 2);
        authenticate_ok(&server, &kp, &jwt, Uuid::new_v4()).await;
        let (unused_fresh, _) = mint(&kp.private, Uuid::new_v4(), 7200);
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            200,
            envelope(refresh_payload(&unused_fresh, Uuid::new_v4())),
        )
        .await;

        let base_url = Url::parse(&server.uri()).unwrap();
        let client = Client::new(&base_url, Uuid::new_v4())
            .unwrap()
            .with_refresh_buffer(Duration::from_secs(1));

        client
            .authenticate("u", "p")
            .await
            .unwrap();
        client.logout().await;

        // the schedule would have fired at login + 1s; give it 4x the lead
        // while the client is very much still alive
        tokio::time::sleep(Duration::from_secs(4)).await;

        assert!(
            received(&server, "POST", REFRESH_PATH)
                .await
                .is_empty(),
            "no background exchange may run after logout"
        );
        assert_eq!(
            received(&server, "GET", KEYS_PATH)
                .await
                .len(),
            1,
            "the aborted task must not even fetch the keyset"
        );
    }

    #[tokio::test]
    async fn auto_refresh_retries_nothing_after_a_failed_exchange() {
        let server = MockServer::start().await;
        let kp = keypair();

        let (jwt, _) = mint(&kp.private, Uuid::new_v4(), 2);
        authenticate_ok(&server, &kp, &jwt, Uuid::new_v4()).await;
        mount_json(
            &server,
            "POST",
            REFRESH_PATH,
            400,
            json!({ "success": false, "errors": ["refresh token has expired"] }),
        )
        .await;

        let base_url = Url::parse(&server.uri()).unwrap();
        let client = Client::new(&base_url, Uuid::new_v4())
            .unwrap()
            .with_refresh_buffer(Duration::from_secs(1));

        client
            .authenticate("u", "p")
            .await
            .unwrap();

        let attempts = wait_for_refresh(&server).await;
        assert_eq!(attempts.len(), 1, "the failed task retires");

        // the client stays alive across the wait, so this pins the failure
        // retirement — not the dropped-client exit: a dead session must not
        // be retried in the background, the request-time path takes over
        tokio::time::sleep(Duration::from_secs(3)).await;
        assert_eq!(
            received(&server, "POST", REFRESH_PATH)
                .await
                .len(),
            1,
            "no hammering of the api on a dead session"
        );
        drop(client);
    }
}
