use std::fmt;
use std::sync::Arc;

use chrono::Utc;
use oxidauth_http::response::Response;
use oxidauth_http::server::api::v1::auth::authenticate::{
    AuthenticateReq, AuthenticateRes,
};
use oxidauth_http::server::api::v1::public_keys::list_all_public_keys::ListAllPublicKeysRes;
use oxidauth_http::server::api::v1::refresh_tokens::exchange::{
    ExchangeRefreshTokenReq, ExchangeRefreshTokenRes,
};
use oxidauth_kernel::authorities::AuthorityStrategy::UsernamePassword;
use oxidauth_kernel::base64::*;
use oxidauth_kernel::jwt::Jwt;
use oxidauth_kernel::public_keys::PublicKey;
use reqwest::header::HeaderMap;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use uuid::Uuid;

mod auth;
mod authorities;
mod can;
mod permissions;
mod public_keys;
mod refresh_tokens;
mod roles;
mod settings;
mod users;

#[derive(Debug, Clone)]
pub struct Client {
    config: Config,
    state: Arc<RwLock<State>>,
}

#[derive(Debug, Clone)]
pub struct Config {
    base_url: String,
    username: String,
    password: String,
}

#[derive(Debug, Default)]
pub struct State {
    client: reqwest::Client,
    jwt: Option<Jwt>,
    refresh_token: Option<Uuid>,
}

impl Client {
    pub fn new(
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, ClientError> {
        Ok(Self {
            config: Config {
                base_url: format!("{}/api/v1", base_url),
                username: username.to_owned(),
                password: password.to_owned(),
            },
            state: Arc::new(RwLock::new(State::default())),
        })
    }

    async fn get_public_keys(&self) -> Result<Vec<PublicKey>, ClientError> {
        let public_keys: Response<ListAllPublicKeysRes> =
            reqwest::Client::new()
                .get(format!(
                    "{}/public_keys",
                    self.config.base_url
                ))
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
                        ClientErrorKind::Other(
                            "unable to deserialize public keys",
                        ),
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
                ))
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

    pub async fn authenticate(&self) -> Result<bool, ClientError> {
        let mut state = self.state.write().await;

        let public_keys = self.get_public_keys().await?;

        // authenticate
        let json = AuthenticateReq {
            strategy: UsernamePassword,
            params: json!({
                "username": self.config.username,
                "password": self.config.password,
            }),
        };

        let response: Response<AuthenticateRes> = reqwest::Client::new()
            .post(format!(
                "{}/auth/authenticate",
                self.config.base_url
            ))
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
                    ClientErrorKind::Other(
                        "unable to deserialize authenticate",
                    ),
                    Some(Box::new(err)),
                )
            })?;

        match response {
            Response {
                success: true,
                payload: Some(payload),
                ..
            } => {
                let mut jwt: Option<Jwt> = None;

                for PublicKey { public_key, .. } in public_keys.into_iter() {
                    let decoded = match BASE64_STANDARD.decode(public_key) {
                        Ok(decoded) => decoded,
                        Err(_) => continue,
                    };

                    if let Ok(decoded_jwt) = Jwt::decode(&payload.jwt, &decoded)
                    {
                        jwt = Some(decoded_jwt);

                        break;
                    }
                }

                match jwt {
                    Some(jwt) => {
                        state.jwt = Some(jwt);
                        state.refresh_token = Some(payload.refresh_token);

                        let bearer = format!("Bearer {}", payload.jwt)
                            .parse()
                            .map_err(|err| {
                                ClientError::new(
                                    ClientErrorKind::Other(
                                        "unable to create bearer token",
                                    ),
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
                                    ClientErrorKind::Other(
                                        "unable to build client in auth",
                                    ),
                                    Some(Box::new(err)),
                                )
                            })?;
                    },
                    None => {
                        return Err(ClientError::new(
                            ClientErrorKind::Other("failed to validate jwt"),
                            None,
                        ));
                    },
                }
            },
            Response {
                success: false,
                errors: Some(errors),
                ..
            } => {
                let errors = serde_json::to_string(&errors).map_err(|err| {
                    ClientError::new(
                        ClientErrorKind::Other(
                            "unable to serialize authenticate errors",
                        ),
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
                ))
            },
        }

        Ok(true)
    }

    pub async fn refresh(&self) -> Result<bool, ClientError> {
        let mut state = self.state.write().await;

        let public_keys = self.get_public_keys().await?;

        let Some(refresh_token) = state.refresh_token else {
            return Err(ClientError::new(
                ClientErrorKind::Other(
                    "can't refresh -- no refresh token found",
                ),
                None,
            ));
        };

        let req = ExchangeRefreshTokenReq { refresh_token };

        let response: Response<ExchangeRefreshTokenRes> =
            reqwest::Client::new()
                .post(format!(
                    "{}/refresh_tokens",
                    self.config.base_url
                ))
                .json(&req)
                .send()
                .await
                .map_err(|err| {
                    ClientError::new(
                        ClientErrorKind::Other(
                            "unable to make request for new refresh token",
                        ),
                        Some(Box::new(err)),
                    )
                })?
                .json()
                .await
                .map_err(|err| {
                    ClientError::new(
                        ClientErrorKind::Other(
                            "unable to make request for new refresh token",
                        ),
                        Some(Box::new(err)),
                    )
                })?;

        match response {
            Response {
                success: true,
                payload: Some(payload),
                ..
            } => {
                let mut jwt: Option<Jwt> = None;

                for PublicKey { public_key, .. } in public_keys.into_iter() {
                    let decoded = match BASE64_STANDARD.decode(public_key) {
                        Ok(decoded) => decoded,
                        Err(_) => continue,
                    };

                    if let Ok(decoded_jwt) = Jwt::decode(&payload.jwt, &decoded)
                    {
                        jwt = Some(decoded_jwt);

                        break;
                    }
                }

                match jwt {
                    Some(jwt) => {
                        state.jwt = Some(jwt);
                        state.refresh_token = Some(payload.refresh_token);

                        let bearer = format!("Bearer {}", payload.jwt)
                            .parse()
                            .map_err(|err| {
                                ClientError::new(
                                    ClientErrorKind::Other(
                                        "unable to create bearer token",
                                    ),
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
                                    ClientErrorKind::Other(
                                        "unable to build client in auth",
                                    ),
                                    Some(Box::new(err)),
                                )
                            })?;
                    },
                    None => {
                        return Err(ClientError::new(
                            ClientErrorKind::Other("failed to validate jwt"),
                            None,
                        ));
                    },
                }
            },
            _ => {
                return Err(ClientError::new(
                    ClientErrorKind::Other(""),
                    None,
                ))
            },
        }

        Ok(true)
    }

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

    async fn authenticate_if_needed(&self) -> Result<bool, ClientError> {
        match self.check_auth_state().await {
            AuthState::Valid => Ok(true),
            AuthState::Auth => self.authenticate().await,
            AuthState::Refresh => match self.refresh().await {
                Ok(res) => Ok(res),
                Err(_) => self.authenticate().await,
            },
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

        let url = format!(
            "{}{}",
            self.config.base_url, url
        );

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
            })?
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

    pub async fn get<Req, Res>(
        &self,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::GET, url, payload)
            .await
    }

    pub async fn put<Req, Res>(
        &self,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::PUT, url, payload)
            .await
    }

    pub async fn post<Req, Res>(
        &self,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
    where
        Req: Serialize + std::fmt::Debug,
        Res: for<'a> Deserialize<'a>,
    {
        self.request(Method::POST, url, payload)
            .await
    }

    pub async fn delete<Req, Res>(
        &self,
        url: &str,
        payload: Req,
    ) -> Result<Res, ClientError>
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
    Authority,
    Role,
    RolePermissionGrant,
    User,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Resource::*;

        match self {
            Authority => write!(f, "authority"),
            Role => write!(f, "role"),
            RolePermissionGrant => write!(f, "role_permission_grant"),
            User => write!(f, "user"),
        }
    }
}

#[derive(Debug)]
pub enum ClientErrorKind {
    AuthError,
    RefreshError,
    EmptyPayload(Resource, &'static str),
    APIResponseError,
    Other(&'static str),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ClientErrorKind::*;

        match self.kind {
            AuthError => write!(
                f,
                "encountered an error authenticating"
            ),
            RefreshError => write!(
                f,
                "encountered an error while refreshing token"
            ),
            EmptyPayload(resource, method) => write!(
                f,
                "received an empty payload when a response payload was expcected for resource {} method {}",
                resource,
                method
            ),
            APIResponseError => write!(f, "error reported when making a request to the API"),
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

fn handle_response<T>(
    resource: Resource,
    method: &'static str,
    response: Response<T>,
) -> Result<T, ClientError>
where
    T: Serialize,
{
    if response.success != true {
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
        .ok_or_else(|| {
            ClientError::new(
                ClientErrorKind::EmptyPayload(resource, method),
                None,
            )
        })?;

    Ok(payload)
}
