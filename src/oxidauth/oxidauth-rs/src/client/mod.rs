pub use std::fmt;
use std::sync::Arc;

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
pub mod settings;
pub mod users;

#[cfg(feature = "mock")]
pub mod mock;

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
    + Send
    + Sync
    + 'static
{
}

#[derive(Debug, Clone)]
pub struct Client {
    config: Config,
    state: Arc<RwLock<State>>,
    #[cfg(feature = "mock")]
    pub mock_jwt: Option<Jwt>,
}

impl ClientTrait for Client {
}

#[cfg(feature = "mock")]
impl ClientTrait for ClientMock {
}

#[derive(Debug, Clone)]
pub struct Config {
    base_url: Url,
    client_key: Uuid,
}

#[derive(Debug, Default)]
pub struct State {
    client: reqwest::Client,
    jwt: Option<Jwt>,
    raw_jwt: Option<String>,
    refresh_token: Option<Uuid>,
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
            },
            state: Arc::new(RwLock::new(State::default())),
            mock_jwt: None,
        });

        #[cfg(not(feature = "mock"))]
        Ok(Self {
            config: Config {
                base_url,
                client_key,
            },
            state: Arc::new(RwLock::new(State::default())),
        })
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
            },
            state: Arc::new(RwLock::new(State::default())),
            mock_jwt: Some(mock_jwt),
        })
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
        let public_keys: Response<ListAllPublicKeysRes> = reqwest::Client::new()
            .get(format!("{}/public_keys", self.config.base_url))
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
                let jwt =
                    Jwt::decode_with_public_keys(&payload.jwt, &public_keys).map_err(|_| {
                        ClientError::new(ClientErrorKind::Other("failed to validate jwt"), None)
                    })?;

                state.raw_jwt = Some(payload.jwt.clone());
                state.jwt = Some(jwt);
                state.refresh_token = Some(payload.refresh_token);

                let bearer = format!("Bearer {}", payload.jwt)
                    .parse()
                    .map_err(|err| {
                        ClientError::new(
                            ClientErrorKind::Other("unable to create bearer token"),
                            Some(Box::new(err)),
                        )
                    })?;

                let mut headers = HeaderMap::new();
                headers.insert("Authorization", bearer);

                state.client = reqwest::Client::builder()
                    .default_headers(headers)
                    .build()
                    .map_err(|err| {
                        ClientError::new(
                            ClientErrorKind::Other("unable to build client in auth"),
                            Some(Box::new(err)),
                        )
                    })?;
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

        Ok(true)
    }

    #[tracing::instrument(skip(self))]
    pub async fn refresh(&self) -> Result<bool, ClientError> {
        let mut state = self.state.write().await;

        let public_keys = self.get_public_keys().await?;

        let Some(refresh_token) = state.refresh_token else {
            return Err(ClientError::new(
                ClientErrorKind::Other("can't refresh -- no refresh token found"),
                None,
            ));
        };

        let req = ExchangeRefreshTokenReq { refresh_token };

        let response: Response<ExchangeRefreshTokenRes> = reqwest::Client::new()
            .post(format!("{}/refresh_tokens", self.config.base_url))
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
                let jwt = Jwt::decode_with_flexible_public_keys(&payload.jwt, &public_keys)
                    .map_err(|_| {
                        ClientError::new(ClientErrorKind::Other("failed to validate jwt"), None)
                    })?;

                state.raw_jwt = Some(payload.jwt.clone());
                state.jwt = Some(jwt);
                state.refresh_token = Some(payload.refresh_token);

                let bearer = format!("Bearer {}", payload.jwt)
                    .parse()
                    .map_err(|err| {
                        ClientError::new(
                            ClientErrorKind::Other("unable to create bearer token"),
                            Some(Box::new(err)),
                        )
                    })?;

                let mut headers = HeaderMap::new();
                headers.insert("Authorization", bearer);

                state.client = reqwest::Client::builder()
                    .default_headers(headers)
                    .build()
                    .map_err(|err| {
                        ClientError::new(
                            ClientErrorKind::Other("unable to build client in auth"),
                            Some(Box::new(err)),
                        )
                    })?;
            },
            _ => return Err(ClientError::new(ClientErrorKind::Other(""), None)),
        }

        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn check_auth_state(&self) -> AuthState {
        let state = self.state.read().await;

        let Some(ref jwt) = state.jwt else {
            return AuthState::Auth;
        };

        let now = Utc::now().timestamp() as usize;

        if now > jwt.exp {
            return AuthState::Refresh;
        }

        AuthState::Valid
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn authenticate_if_needed(&self) -> Result<bool, ClientError> {
        match self.check_auth_state().await {
            AuthState::Valid => Ok(true),
            AuthState::Auth => Ok(false),
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

        let res = client
            .request(method, url)
            .json(&payload)
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
                write!(f, "no jwt found when calling get_jwt")
            },
            AuthError => {
                write!(f, "encountered an error authenticating")
            },
            RefreshError => {
                write!(f, "encountered an error while refreshing token")
            },
            EmptyPayload(resource, method) => {
                write!(
                    f,
                    "received an empty payload when a response payload was expcected for resource {} method {}",
                    resource, method
                )
            },
            APIResponseError => {
                write!(f, "error reported when making a request to the API")
            },
            UrlParseError => {
                write!(f, "encountered an error while parsing url")
            },
            Other(reason) => write!(f, "error: {}", reason),
        }
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
        assert_eq!(err.to_string(), "encountered an error while parsing url");
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
}
