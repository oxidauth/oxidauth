pub use std::fmt;
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

use oxidauth_kernel::jwt::Jwt;
use oxidauth_kernel::public_keys::PublicKey;
use oxidauth_kernel::{base64::*, JsonValue, Password};
use reqwest::header::HeaderMap;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::info;
use url::Url;
use uuid::Uuid;

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

#[derive(Debug, Clone)]
pub struct Client {
    config: Config,
    state: Arc<RwLock<State>>,
    #[cfg(feature = "mock")]
    pub mock_jwt: Option<Jwt>,
}

#[derive(Debug, Clone)]
pub struct Config {
    base_url: Url,
    client_key: Uuid,
    username: String,
    password: Password,
}

#[derive(Debug, Default)]
pub struct State {
    client: reqwest::Client,
    jwt: Option<Jwt>,
    raw_jwt: Option<String>,
    refresh_token: Option<Uuid>,
}

impl Client {
    pub fn new(
        base_url: &Url,
        client_key: Uuid,
        username: &str,
        password: &str,
    ) -> Result<Self, ClientError> {
        let base_url = base_url
            .join("/api/v1")
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::UrlParseError,
                    Some(Box::new(err)),
                )
            })?;

        #[cfg(feature = "mock")]
        return Ok(Self {
            config: Config {
                base_url,
                client_key,
                username: username.to_owned(),
                password: Password::new(password.to_owned()),
            },
            state: Arc::new(RwLock::new(State::default())),
            mock_jwt: None,
        });

        #[cfg(not(feature = "mock"))]
        Ok(Self {
            config: Config {
                base_url,
                client_key,
                username: username.to_owned(),
                password: Password::new(password.to_owned()),
            },
            state: Arc::new(RwLock::new(State::default())),
        })
    }

    #[cfg(feature = "mock")]
    pub fn test_client(
        mock_jwt: Jwt,
    ) -> Result<Self, ClientError> {
        let base_url = Url::parse("http://base_url.com/")
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::UrlParseError,
                    Some(Box::new(err)),
                )
            })?
            .join("/api/v1")
            .map_err(|err| {
                ClientError::new(
                    ClientErrorKind::UrlParseError,
                    Some(Box::new(err)),
                )
            })?;

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

    pub async fn get_jwt(&self) -> Result<String, ClientError> {
        self.authenticate_if_needed()
            .await?;

        let state = self.state.read().await;

        let jwt = state
            .raw_jwt
            .as_deref()
            .ok_or(ClientError::new(
                ClientErrorKind::NoJwtFound,
                None,
            ))?;

        Ok(jwt.to_string())
    }

    pub async fn get_jwt_decoded(&self) -> Result<Jwt, ClientError> {
        self.authenticate_if_needed()
            .await?;

        let state = self.state.read().await;

        let jwt = state
            .jwt
            .clone()
            .ok_or(ClientError::new(
                ClientErrorKind::NoJwtFound,
                None,
            ))?;

        Ok(jwt)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_public_keys(&self) -> Result<Vec<PublicKey>, ClientError> {
        #[cfg(feature = "mock")]
        return Ok(vec![PublicKey {
            id: Uuid::new_v4(),
            public_key: "public_key".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }]);

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
    async fn auth(&self) -> Result<bool, ClientError> {
        #[cfg(feature = "mock")]
        dbg!("HELLOO>>");

        #[cfg(feature = "mock")]
        return Ok(true);

        let mut state = self.state.write().await;

        let public_keys = self.get_public_keys().await?;

        // authenticate
        let json = AuthenticateReq {
            client_key: self.config.client_key,
            params: JsonValue::new(json!({
                "username": self.config.username,
                "password": self.config.password,
            })),
        };

        info!(message = "authenticating", params = ?json);

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
                let jwt =
                    Jwt::decode_with_public_keys(&payload.jwt, &public_keys)
                        .map_err(|_| {
                            ClientError::new(
                                ClientErrorKind::Other(
                                    "failed to validate jwt",
                                ),
                                None,
                            )
                        })?;

                state.raw_jwt = Some(payload.jwt.clone());
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
                ));
            },
        }

        Ok(true)
    }

    #[tracing::instrument(skip(self))]
    pub async fn refresh(&self) -> Result<bool, ClientError> {
        #[cfg(feature = "mock")]
        dbg!("HELLOO>>");

        #[cfg(feature = "mock")]
        return Ok(true);

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
                ));
            },
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
            AuthState::Auth => self.auth().await,
            AuthState::Refresh => match self.refresh().await {
                Ok(res) => Ok(res),
                Err(_) => self.auth().await,
            },
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
            NoJwtFound => write!(f, "no jwt found when calling get_jwt"),
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
            UrlParseError => write!(f, "encountered an error while parsing url"),
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
        .ok_or_else(|| {
            ClientError::new(
                ClientErrorKind::EmptyPayload(resource, method),
                None,
            )
        })?;

    Ok(payload)
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::sync::Arc;
    use uuid::Uuid;
    use oxidauth_kernel::error::BoxedError;

    // Users
    use oxidauth_http::server::api::v1::users::create_user::{CreateUserReq, CreateUserRes};
    use oxidauth_http::server::api::v1::users::delete_user_by_id::DeleteUserByIdRes;
    use oxidauth_http::server::api::v1::users::find_user_by_id::{FindUserByIdReq, FindUserByIdRes};
    use oxidauth_http::server::api::v1::users::find_user_by_username::FindUserByUsernameRes;
    use oxidauth_http::server::api::v1::users::find_users_by_ids::{FindUsersByIdsReq, FindUsersByIdsRes};
    use oxidauth_http::server::api::v1::users::list_all_users::{ListAllUsersReq, ListAllUsersRes};
    use oxidauth_http::server::api::v1::users::update_user::{UpdateUserBodyReq, UpdateUserRes};

    // User Authorities
    use oxidauth_http::server::api::v1::users::authorities::create_user_authority::{CreateUserAuthorityBodyReq, CreateUserAuthorityRes};
    use oxidauth_http::server::api::v1::users::authorities::delete_user_authority::{DeleteUserAuthorityReq, DeleteUserAuthorityRes};
    use oxidauth_http::server::api::v1::users::authorities::find_user_authority_by_user_id_and_authority_id::{FindUserAuthorityByUserIdAndAuthorityIdReq, FindUserAuthorityByUserIdAndAuthorityIdRes};
    use oxidauth_http::server::api::v1::users::authorities::list_user_authorities_by_user_id::{ListUserAuthoritiesByUserIdReq, ListUserAuthoritiesByUserIdRes};
    use oxidauth_http::server::api::v1::users::authorities::update_user_authority::UpdateUserAuthorityRes;
    use oxidauth_kernel::user_authorities::update_user_authority::UpdateUserAuthority;

    // User Permissions
    use oxidauth_http::server::api::v1::users::permissions::create_user_permission::{CreateUserPermissionReq, CreateUserPermissionRes};
    use oxidauth_http::server::api::v1::users::permissions::delete_user_permission::{DeleteUserPermissionReq, DeleteUserPermissionRes};
    use oxidauth_http::server::api::v1::users::permissions::list_user_permissions_by_user_id::{ListUserPermissionGrantsByUserIdReq, ListUserPermissionGrantsByUserIdRes};

    // User Roles
    use crate::client::users::roles::create_user_role::CreateUserRole;
    use oxidauth_http::server::api::v1::users::roles::create_user_role::CreateUserRoleRes;
    use oxidauth_http::server::api::v1::users::roles::delete_user_role::DeleteUserRoleRes;
    use oxidauth_http::server::api::v1::users::roles::list_user_roles_by_user_id::ListUserRoleGrantsByUserIdRes;

    // Auth
    use oxidauth_http::server::api::v1::auth::register::{RegisterReq, RegisterRes};

    // Can
    use oxidauth_http::server::api::v1::can::CanReq;

    // Authorities
    use oxidauth_http::server::api::v1::authorities::create_authority::{CreateAuthorityReq, CreateAuthorityRes};
    use oxidauth_http::server::api::v1::authorities::delete_authority::DeleteAuthorityRes;
    use oxidauth_http::server::api::v1::authorities::find_authority_by_id::FindAuthorityByIdRes;
    use oxidauth_http::server::api::v1::authorities::find_authority_by_strategy::FindAuthorityByStrategyRes;
    use oxidauth_http::server::api::v1::authorities::list_all_authorities::{ListAllAuthoritiesReq, ListAllAuthoritiesRes};
    use oxidauth_http::server::api::v1::authorities::update_authority::{UpdateAuthorityReq, UpdateAuthorityRes};

    // Permissions
    use oxidauth_http::server::api::v1::permissions::create_permission::{CreatePermissionReq, CreatePermissionRes};
    use oxidauth_http::server::api::v1::permissions::delete_permission::{DeletePermissionReq, DeletePermissionRes};
    use oxidauth_http::server::api::v1::permissions::find_permission_by_parts::{FindPermissionByPartsReq, FindPermissionByPartsRes};
    use oxidauth_http::server::api::v1::permissions::list_all_permissions::{ListAllPermissionsReq, ListAllPermissionsRes};

    // Roles
    use oxidauth_http::server::api::v1::roles::create_role::{CreateRoleReq, CreateRoleRes};
    use oxidauth_http::server::api::v1::roles::delete_role::DeleteRoleRes;
    use oxidauth_http::server::api::v1::roles::find_role_by_id::FindRoleByIdRes;
    use oxidauth_http::server::api::v1::roles::find_role_by_name::FindRoleByNameRes;
    use oxidauth_http::server::api::v1::roles::list_all_roles::{ListAllRolesReq, ListAllRolesRes};
    use oxidauth_http::server::api::v1::roles::update_role::{UpdateRoleReq, UpdateRoleRes};

    // Role Permission Grants
    use oxidauth_http::server::api::v1::roles::permissions::create_role_permission_grant::{CreateRolePermissionGrantReq, CreateRolePermissionGrantRes};
    use oxidauth_http::server::api::v1::roles::permissions::delete_role_permission_grant::{DeleteRolePermissionGrantReq, DeleteRolePermissionGrantRes};
    use oxidauth_http::server::api::v1::roles::permissions::list_role_permission_grants_by_role_id::{ListRolePermissionGrantsByRoleIdReq, ListRolePermissionGrantsByRoleIdRes};

    // Role Role Grants
    use oxidauth_http::server::api::v1::roles::roles::create_role_role_grant::{CreateRoleRoleGrantReq, CreateRoleRoleGrantRes};
    use oxidauth_http::server::api::v1::roles::roles::delete_role_role_grant::{DeleteRoleRoleGrantReq, DeleteRoleRoleGrantRes};
    use oxidauth_http::server::api::v1::roles::roles::list_role_role_grants_by_parent_id::{ListRoleRoleGrantsByParentIdReq, ListRoleRoleGrantsByParentIdRes};

    // Public Keys
    use oxidauth_http::server::api::v1::public_keys::create_public_key::CreatePublicKeyRes;
    use oxidauth_http::server::api::v1::public_keys::delete_public_key::DeletePublicKeyRes;
    use oxidauth_http::server::api::v1::public_keys::find_public_key_by_id::FindPublicKeyByIdRes;
    use oxidauth_http::server::api::v1::public_keys::list_all_public_keys::ListAllPublicKeysRes;

    // Refresh Tokens
    use oxidauth_http::server::api::v1::refresh_tokens::exchange::{ExchangeRefreshTokenReq, ExchangeRefreshTokenRes};

    // Settings
    use oxidauth_http::server::api::v1::settings::fetch_setting::{FetchSettingReq, FetchSettingRes};
    use oxidauth_http::server::api::v1::settings::save_setting::{SaveSettingReq, SaveSettingRes};

    // Invitations
    use oxidauth_http::server::api::v1::invitations::accept_invitation::{AcceptInvitationParams, AcceptInvitationRes};
    use oxidauth_http::server::api::v1::invitations::create_invitation::{CreateInvitationReq, CreateInvitationRes};
    use oxidauth_http::server::api::v1::invitations::find_invitation::{FindInvitationReq, FindInvitationRes};

    #[derive(Default)]
    pub struct ClientMock {
        // Users
        pub list_all_users_fn: Option<Arc<dyn Fn(ListAllUsersReq) -> Result<ListAllUsersRes, BoxedError> + Send + Sync>>,
        pub create_user_fn: Option<Arc<dyn Fn(CreateUserReq) -> Result<CreateUserRes, BoxedError> + Send + Sync>>,
        pub delete_user_fn: Option<Arc<dyn Fn(Uuid) -> Result<DeleteUserByIdRes, BoxedError> + Send + Sync>>,
        pub find_user_by_id_fn: Option<Arc<dyn Fn(FindUserByIdReq) -> Result<FindUserByIdRes, BoxedError> + Send + Sync>>,
        pub find_user_by_username_fn: Option<Arc<dyn Fn(String) -> Result<FindUserByUsernameRes, BoxedError> + Send + Sync>>,
        pub find_users_by_ids_fn: Option<Arc<dyn Fn(FindUsersByIdsReq) -> Result<FindUsersByIdsRes, BoxedError> + Send + Sync>>,
        pub update_user_fn: Option<Arc<dyn Fn(Uuid, UpdateUserBodyReq) -> Result<UpdateUserRes, BoxedError> + Send + Sync>>,

        // User Authorities
        pub create_user_authority_fn: Option<Arc<dyn Fn(Uuid, CreateUserAuthorityBodyReq) -> Result<CreateUserAuthorityRes, BoxedError> + Send + Sync>>,
        pub delete_user_authority_fn: Option<Arc<dyn Fn(DeleteUserAuthorityReq) -> Result<DeleteUserAuthorityRes, BoxedError> + Send + Sync>>,
        pub find_user_authority_by_user_id_and_authority_id_fn: Option<Arc<dyn Fn(FindUserAuthorityByUserIdAndAuthorityIdReq) -> Result<FindUserAuthorityByUserIdAndAuthorityIdRes, BoxedError> + Send + Sync>>,
        pub list_user_authorities_by_user_id_fn: Option<Arc<dyn Fn(ListUserAuthoritiesByUserIdReq) -> Result<ListUserAuthoritiesByUserIdRes, BoxedError> + Send + Sync>>,
        pub update_user_authority_fn: Option<Arc<dyn Fn(UpdateUserAuthority) -> Result<UpdateUserAuthorityRes, BoxedError> + Send + Sync>>,

        // User Permissions
        pub create_user_permission_grant_fn: Option<Arc<dyn Fn(CreateUserPermissionReq) -> Result<CreateUserPermissionRes, BoxedError> + Send + Sync>>,
        pub delete_user_permission_grant_fn: Option<Arc<dyn Fn(DeleteUserPermissionReq) -> Result<DeleteUserPermissionRes, BoxedError> + Send + Sync>>,
        pub list_user_permission_grants_by_user_id_fn: Option<Arc<dyn Fn(ListUserPermissionGrantsByUserIdReq) -> Result<ListUserPermissionGrantsByUserIdRes, BoxedError> + Send + Sync>>,

        // User Roles
        pub create_user_role_fn: Option<Arc<dyn Fn(CreateUserRole) -> Result<CreateUserRoleRes, BoxedError> + Send + Sync>>,
        pub delete_user_role_fn: Option<Arc<dyn Fn(Uuid, Uuid) -> Result<DeleteUserRoleRes, BoxedError> + Send + Sync>>,
        pub list_user_roles_by_user_id_fn: Option<Arc<dyn Fn(Uuid) -> Result<ListUserRoleGrantsByUserIdRes, BoxedError> + Send + Sync>>,

        // Auth
        pub authenticate_fn: Option<Arc<dyn Fn() -> Result<bool, BoxedError> + Send + Sync>>,
        pub register_fn: Option<Arc<dyn Fn(RegisterReq) -> Result<RegisterRes, BoxedError> + Send + Sync>>,

        // Can
        pub can_fn: Option<Arc<dyn Fn(CanReq) -> Result<bool, BoxedError> + Send + Sync>>,

        // Authorities
        pub create_authority_fn: Option<Arc<dyn Fn(CreateAuthorityReq) -> Result<CreateAuthorityRes, BoxedError> + Send + Sync>>,
        pub delete_authority_fn: Option<Arc<dyn Fn(Uuid) -> Result<DeleteAuthorityRes, BoxedError> + Send + Sync>>,
        pub find_authority_by_id_fn: Option<Arc<dyn Fn(Uuid) -> Result<FindAuthorityByIdRes, BoxedError> + Send + Sync>>,
        pub find_authority_by_strategy_fn: Option<Arc<dyn Fn(String) -> Result<FindAuthorityByStrategyRes, BoxedError> + Send + Sync>>,
        pub list_all_authorities_fn: Option<Arc<dyn Fn(ListAllAuthoritiesReq) -> Result<ListAllAuthoritiesRes, BoxedError> + Send + Sync>>,
        pub update_authority_fn: Option<Arc<dyn Fn(Uuid, UpdateAuthorityReq) -> Result<UpdateAuthorityRes, BoxedError> + Send + Sync>>,

        // Permissions
        pub create_permission_fn: Option<Arc<dyn Fn(CreatePermissionReq) -> Result<CreatePermissionRes, BoxedError> + Send + Sync>>,
        pub delete_permission_fn: Option<Arc<dyn Fn(DeletePermissionReq) -> Result<DeletePermissionRes, BoxedError> + Send + Sync>>,
        pub find_permission_by_parts_fn: Option<Arc<dyn Fn(FindPermissionByPartsReq) -> Result<FindPermissionByPartsRes, BoxedError> + Send + Sync>>,
        pub list_all_permissions_fn: Option<Arc<dyn Fn(ListAllPermissionsReq) -> Result<ListAllPermissionsRes, BoxedError> + Send + Sync>>,

        // Roles
        pub create_role_fn: Option<Arc<dyn Fn(CreateRoleReq) -> Result<CreateRoleRes, BoxedError> + Send + Sync>>,
        pub delete_role_fn: Option<Arc<dyn Fn(Uuid) -> Result<DeleteRoleRes, BoxedError> + Send + Sync>>,
        pub find_role_by_id_fn: Option<Arc<dyn Fn(Uuid) -> Result<FindRoleByIdRes, BoxedError> + Send + Sync>>,
        pub find_role_by_name_fn: Option<Arc<dyn Fn(String) -> Result<FindRoleByNameRes, BoxedError> + Send + Sync>>,
        pub list_all_roles_fn: Option<Arc<dyn Fn(ListAllRolesReq) -> Result<ListAllRolesRes, BoxedError> + Send + Sync>>,
        pub update_role_fn: Option<Arc<dyn Fn(Uuid, UpdateRoleReq) -> Result<UpdateRoleRes, BoxedError> + Send + Sync>>,

        // Role Permission Grants
        pub create_role_permission_grant_fn: Option<Arc<dyn Fn(CreateRolePermissionGrantReq) -> Result<CreateRolePermissionGrantRes, BoxedError> + Send + Sync>>,
        pub delete_role_permission_grant_fn: Option<Arc<dyn Fn(DeleteRolePermissionGrantReq) -> Result<DeleteRolePermissionGrantRes, BoxedError> + Send + Sync>>,
        pub list_role_permission_grants_by_role_id_fn: Option<Arc<dyn Fn(ListRolePermissionGrantsByRoleIdReq) -> Result<ListRolePermissionGrantsByRoleIdRes, BoxedError> + Send + Sync>>,

        // Role Role Grants
        pub create_role_role_grant_fn: Option<Arc<dyn Fn(CreateRoleRoleGrantReq) -> Result<CreateRoleRoleGrantRes, BoxedError> + Send + Sync>>,
        pub delete_role_role_grant_fn: Option<Arc<dyn Fn(DeleteRoleRoleGrantReq) -> Result<DeleteRoleRoleGrantRes, BoxedError> + Send + Sync>>,
        pub list_role_role_grants_by_parent_id_fn: Option<Arc<dyn Fn(ListRoleRoleGrantsByParentIdReq) -> Result<ListRoleRoleGrantsByParentIdRes, BoxedError> + Send + Sync>>,

        // Public Keys
        pub create_public_key_fn: Option<Arc<dyn Fn() -> Result<CreatePublicKeyRes, BoxedError> + Send + Sync>>,
        pub delete_public_key_fn: Option<Arc<dyn Fn(Uuid) -> Result<DeletePublicKeyRes, BoxedError> + Send + Sync>>,
        pub find_public_key_by_id_fn: Option<Arc<dyn Fn(Uuid) -> Result<FindPublicKeyByIdRes, BoxedError> + Send + Sync>>,
        pub list_all_public_keys_fn: Option<Arc<dyn Fn() -> Result<ListAllPublicKeysRes, BoxedError> + Send + Sync>>,

        // Refresh Tokens
        pub exchange_refresh_token_fn: Option<Arc<dyn Fn(ExchangeRefreshTokenReq) -> Result<ExchangeRefreshTokenRes, BoxedError> + Send + Sync>>,

        // Settings
        pub fetch_setting_fn: Option<Arc<dyn Fn(FetchSettingReq) -> Result<FetchSettingRes, BoxedError> + Send + Sync>>,
        pub save_setting_fn: Option<Arc<dyn Fn(SaveSettingReq) -> Result<SaveSettingRes, BoxedError> + Send + Sync>>,

        // Invitations
        pub accept_invitation_fn: Option<Arc<dyn Fn(AcceptInvitationParams) -> Result<AcceptInvitationRes, BoxedError> + Send + Sync>>,
        pub create_invitation_fn: Option<Arc<dyn Fn(CreateInvitationReq) -> Result<CreateInvitationRes, BoxedError> + Send + Sync>>,
        pub find_invitation_fn: Option<Arc<dyn Fn(FindInvitationReq) -> Result<FindInvitationRes, BoxedError> + Send + Sync>>,
    }
}
