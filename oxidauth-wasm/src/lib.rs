pub mod auth;
pub mod can;
pub mod jwt;
pub mod public_keys;
pub mod refresh_tokens;
pub mod response;
pub mod totp;

pub use std::fmt;
use std::{error::Error, sync::Arc, time::Duration};

use chrono::Utc;
use gloo_storage::{LocalStorage, Storage};
use log::info;
use reqwest::{Method, header::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::json;

use auth::{AuthenticateReq, AuthenticateRes};
use tokio::{sync::RwLock, time::sleep};
use wasm_bindgen_futures::spawn_local;

use url::Url;
use uuid::Uuid;

#[cfg(feature = "mock")]
use crate::mock::ClientMock;

pub use crate::{
    auth::AuthTrait, can::CanTrait, public_keys::PublicKeysTrait,
    refresh_tokens::RefreshTokensTrait,
};
use crate::{
    jwt::get_jwt_exp_no_decode,
    public_keys::{ListAllPublicKeysRes, PublicKey},
    refresh_tokens::{ExchangeRefreshTokenReq, ExchangeRefreshTokenRes, ExchangeRefreshTokenTrait},
    response::Response,
};

#[cfg(feature = "mock")]
pub mod mock;

pub trait ClientTrait:
    AuthTrait + CanTrait + PublicKeysTrait + RefreshTokensTrait + Sync + 'static
{
}

pub type BoxedError = Box<dyn Error + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct Client {
    config: Config,
    state: Arc<RwLock<State>>,
    #[cfg(feature = "mock")]
    pub mock_jwt: Option<Jwt>,
}

impl ClientTrait for Client {}

#[cfg(feature = "mock")]
impl ClientTrait for ClientMock {}

#[derive(Debug, Clone)]
pub struct Config {
    base_url: Url,
    client_key: Uuid,
}

#[derive(Debug, Default)]
pub struct State {
    client: reqwest::Client,
    // jwt: Option<Jwt>,
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

        let raw_jwt: Option<String> = LocalStorage::get("OXIDAUTH_TOKEN").ok();
        let refresh_token = LocalStorage::get("OXIDAUTH_REFRESH_TOKEN").ok();

        let mut headers = HeaderMap::new();

        if let Some(jwt_token) = &raw_jwt {
            let bearer = format!("Bearer {}", jwt_token)
                .parse()
                .map_err(|err| {
                    ClientError::new(
                        ClientErrorKind::Other("unable to create bearer token"),
                        Some(Box::new(err)),
                    )
                })?;

            headers.insert("Authorization", bearer);
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::Other("unable to build client in auth"),
                    Some(Box::new(err)),
                )
            })?;

        #[cfg(not(feature = "mock"))]
        let client = Self {
            config: Config {
                base_url,
                client_key,
            },
            state: Arc::new(RwLock::new(State {
                client,
                raw_jwt,
                refresh_token,
            })),
        };

        // start automatic refresh token exchange
        #[cfg(not(feature = "mock"))]
        let _client = client.clone();

        #[cfg(not(feature = "mock"))]
        spawn_local(async move {
            _client
                .timed_refresh_token_exchange()
                .await
        });

        #[cfg(not(feature = "mock"))]
        Ok(client)
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
                username: "username".to_owned(),
                password: Password::new("password".to_owned()),
            },
            state: Arc::new(RwLock::new(State::default())),
            mock_jwt: Some(mock_jwt),
        })
    }

    pub async fn timed_refresh_token_exchange(&self) {
        info!("starting timed refresh loop");

        loop {
            info!("top of timed refresh loop");
            let state = self.state.read().await;
            // let Ok(public_keys) = self.get_public_keys().await else {
            //     ClientError::new(ClientErrorKind::Other("could not get public keys"), None);
            //     return;
            // };

            // get jwt
            // let Some(jwt) = state.jwt.clone() else {
            //     ClientError::new(
            //         ClientErrorKind::Other("unable to fetch JWT from state"),
            //         None,
            //     );
            //     return;
            // };

            let Some(jwt_exp) = get_jwt_exp_no_decode(state.raw_jwt.clone()) else {
                ClientError::new(
                    ClientErrorKind::Other("unable to decode jwt to get exp time"),
                    None,
                );

                continue;
            };

            info!(
                "!! #1 - JWT exp {} - now: {}",
                jwt_exp,
                Utc::now().timestamp()
            );

            let ten_seconds_from_now = Utc::now().timestamp() as usize + 10;

            if jwt_exp < ten_seconds_from_now {
                info!("found exp within 10 secs - refreshing tokens");

                let Some(refresh_token) = state.refresh_token else {
                    ClientError::new(
                        ClientErrorKind::Other("unable to fetch refresh token from state"),
                        None,
                    );

                    continue;
                };

                drop(state);

                info!("exchanging token");

                let Ok(res) = self
                    .exchange_refresh_token(ExchangeRefreshTokenReq { refresh_token })
                    .await
                else {
                    info!("failed to exchange token");
                    ClientError::new(ClientErrorKind::Other("failed to exchange token"), None);

                    continue;
                };

                info!("exchange token complete");

                // TODO
                // let Ok(jwt) = Jwt::decode_with_public_keys(&res.jwt, &public_keys).map_err(|_| {
                //     info!("failed to validate jwt");
                //     ClientError::new(ClientErrorKind::Other("failed to validate jwt"), None)
                // }) else {
                //     info!("failed to decode jwt");
                //     ClientError::new(ClientErrorKind::Other("failed to decode jwt"), None);
                //     return;
                // };

                let mut state = self.state.write().await;

                info!("!! #2 - JWT exp {}", jwt_exp);

                info!("writing new tokens to state");
                state.raw_jwt = Some(res.jwt.clone());
                // state.jwt = Some(jwt);
                state.refresh_token = Some(res.refresh_token);

                let Ok(bearer) = format!("Bearer {}", res.jwt.clone())
                    .parse()
                    .map_err(|err| {
                        ClientError::new(
                            ClientErrorKind::Other("unable to create bearer token"),
                            Some(Box::new(err)),
                        )
                    })
                else {
                    continue;
                };

                let mut headers = HeaderMap::new();
                headers.insert("Authorization", bearer);

                let Ok(client) = reqwest::Client::builder()
                    .default_headers(headers)
                    .build()
                    .map_err(|err| {
                        ClientError::new(
                            ClientErrorKind::Other("unable to build client in auth"),
                            Some(Box::new(err)),
                        )
                    })
                else {
                    continue;
                };

                state.client = client;

                let _ = LocalStorage::set("OXIDAUTH_TOKEN", res.jwt.clone());
                let _ = LocalStorage::set("OXIDAUTH_REFRESH_TOKEN", res.refresh_token);

                drop(state);
            }

            sleep(Duration::from_secs(2)).await;
        }
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

    pub async fn authenticate(
        &self,
        client_key: Uuid,
        username: &str,
        password: &str,
    ) -> Result<bool, ClientError> {
        self.auth(client_key, username, password)
            .await
    }

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

    async fn auth(
        &self,
        client_key: Uuid,
        username: &str,
        password: &str,
    ) -> Result<bool, ClientError> {
        let mut state = self.state.write().await;

        // let public_keys = self.get_public_keys().await?;

        // authenticate
        let json = AuthenticateReq {
            client_key,
            params: json!({
                "username": username,
                "password": password,
            }),
        };

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
                // TODO - For verify step
                // let jwt =
                //     Jwt::decode_with_public_keys(&payload.jwt, &public_keys).map_err(|_| {
                //         ClientError::new(ClientErrorKind::Other("failed to validate jwt"), None)
                //     })?;

                state.raw_jwt = Some(payload.jwt.clone());
                // state.jwt = Some(jwt);
                state.refresh_token = Some(payload.refresh_token);

                let _ = LocalStorage::set("OXIDAUTH_TOKEN", payload.jwt.clone());
                let _ = LocalStorage::set("OXIDAUTH_REFRESH_TOKEN", payload.refresh_token);

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
                    None,
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

    pub async fn refresh(&self) -> Result<bool, ClientError> {
        let mut state = self.state.write().await;

        // let public_keys = self.get_public_keys().await?;

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
                // TODO FOR VERIFY STEP
                // let mut jwt: Option<Jwt> = None;

                // for PublicKey { public_key, .. } in public_keys.into_iter() {
                //     let decoded = match BASE64_STANDARD.decode(public_key) {
                //         Ok(decoded) => decoded,
                //         Err(_) => continue,
                //     };

                //     if let Ok(decoded_jwt) = Jwt::decode(&payload.jwt, &decoded) {
                //         jwt = Some(decoded_jwt);

                //         break;
                //     }
                // }

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

    async fn check_auth_state(&self) -> AuthState {
        let state = self.state.read().await;

        // let Some(ref _jwt) = state.raw_jwt else {
        //     return AuthState::Auth;
        // };

        let now = Utc::now().timestamp() as usize;

        let Some(jwt_exp) = get_jwt_exp_no_decode(state.raw_jwt.clone()) else {
            return AuthState::Auth;
        };

        if now > jwt_exp {
            return AuthState::Refresh;
        }

        AuthState::Valid
    }

    async fn authenticate_if_needed(&self) -> Result<bool, ClientError> {
        match self.check_auth_state().await {
            AuthState::Valid => Ok(true),
            AuthState::Auth => Ok(false),
            AuthState::Refresh => self.refresh().await,
        }
    }

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

        let mut req = client.request(method.clone(), url);

        if method != Method::GET {
            req = req.json(&payload);
        }

        let res = req
            .send()
            .await
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::OtherString(err.to_string()),
                    Some(Box::new(err)),
                )
            })?;

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
    pub source: Option<Box<dyn std::error::Error + Sync + 'static>>,
}

impl ClientError {
    pub fn new(
        kind: ClientErrorKind,
        source: Option<Box<dyn std::error::Error + Sync + 'static>>,
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
    OtherString(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ClientErrorKind::*;

        match &self.kind {
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
            OtherString(reason) => write!(f, "error: {}", reason),
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
            source: None,
        });
    }

    let payload = response
        .payload
        .ok_or_else(|| ClientError::new(ClientErrorKind::EmptyPayload(resource, method), None))?;

    Ok(payload)
}
