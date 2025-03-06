use std::sync::Arc;

use async_trait::async_trait;

use oxidauth_kernel::{
    JsonValue,
    auth::{
        authenticate::{AuthenticateParams, AuthenticateResponse},
        register::RegisterParams,
    },
    error::BoxedError,
    service::Service,
};
use oxidauth_repository::{
    auth::tree::PermissionTreeQuery,
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    private_keys::select_most_recent_private_key::SelectMostRecentPrivateKeyQuery,
    refresh_tokens::insert_refresh_token::InsertRefreshTokenQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    user_authorities::{
        insert_user_authority::InsertUserAuthorityQuery,
        select_user_authorities_by_authority_id_and_user_identifier::SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    },
    users::{insert_user::InsertUserQuery, select_user_by_id_query::SelectUserByIdQuery},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{authenticate::AuthenticateUseCase, register::RegisterUseCase};

// TODO(dewey4iv): move to kernel
pub type AuthenticateOrRegisterService = Arc<
    dyn for<'a> Service<
            &'a AuthenticateOrRegisterParams,
            Response = AuthenticateResponse,
            Error = BoxedError,
        >,
>;

pub struct AuthenticateOrRegisterUseCase<A, M, P, R, S, T, UI, U, UU>
where
    A: InsertUserAuthorityQuery,
    M: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    UI: InsertUserQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    UU: SelectUserByIdQuery,
{
    authenticate: AuthenticateUseCase<T, U, P, M, R, S, UU>,
    register: RegisterUseCase<T, UI, A, P, M, R>,
}

#[async_trait]
impl<'a, A, M, P, R, S, T, UI, U, UU> Service<&'a AuthenticateOrRegisterParams>
    for AuthenticateOrRegisterUseCase<A, M, P, R, S, T, UI, U, UU>
where
    A: InsertUserAuthorityQuery,
    M: SelectMostRecentPrivateKeyQuery,
    P: PermissionTreeQuery,
    R: InsertRefreshTokenQuery,
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    UI: InsertUserQuery,
    U: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
    UU: SelectUserByIdQuery,
{
    type Response = AuthenticateResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "authenticate_or_register_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a AuthenticateOrRegisterParams,
    ) -> Result<Self::Response, Self::Error> {
        let authenticated = self
            .authenticate
            .call(&params.into())
            .await;

        match authenticated {
            Ok(auth) => return Ok(auth),
            Err(err) => {
                if err
                    .to_string()
                    .contains("user authority not found:")
                {
                    self.register
                        .call(&params.into())
                        .await?;
                }

                return Err(err);
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateOrRegisterParams {
    pub client_key: Uuid,
    pub params: JsonValue,
}

impl From<&AuthenticateOrRegisterParams> for AuthenticateParams {
    fn from(value: &AuthenticateOrRegisterParams) -> Self {
        Self {
            client_key: value.client_key,
            params: value.params.clone(),
        }
    }
}

impl From<&AuthenticateOrRegisterParams> for RegisterParams {
    fn from(value: &AuthenticateOrRegisterParams) -> Self {
        Self {
            client_key: value.client_key,
            params: value.params.clone(),
        }
    }
}

//impl AuthenticateOrRegisterUsecase {
//    pub fn new() -> Self {
//        Self {}
//    }
//}
