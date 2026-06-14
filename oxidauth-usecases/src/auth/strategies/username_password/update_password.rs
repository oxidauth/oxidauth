use async_trait::async_trait;
use boringauth::oath::TOTPBuilder;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

use oxidauth_kernel::{
    JsonValue,
    auth::username_password::update_password::{UpdatePasswordParams, UpdatePasswordResponse},
    authorities::find_authority_by_client_key::FindAuthorityByClientKey,
    error::BoxedError,
    service::Service,
    totp_secrets::find_totp_secret_by_user_id::FindTOTPSecretByUserId,
    user_authorities::update_user_authority::UpdateUserAuthority,
};
use oxidauth_repository::{
    authorities::select_authority_by_client_key::SelectAuthorityByClientKeyQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
    user_authorities::{
        select_user_authorities_by_authority_id_and_user_identifier::{
            SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
            SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams,
        },
        update_user_authority::UpdateUserAuthorityQuery,
    },
};

use crate::auth::strategies::username_password::{AuthorityParams, UserAuthorityParams};

use super::helpers::{hash_password, raw_password_hash};

pub struct UpdatePasswordUseCase<S, T, U, V>
where
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    U: UpdateUserAuthorityQuery,
    V: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
{
    user_totp_secret: S,
    authority_by_client_key: T,
    update_user_authority: U,
    select_user_authority: V,
}

impl<S, T, U, V> UpdatePasswordUseCase<S, T, U, V>
where
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    U: UpdateUserAuthorityQuery,
    V: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
{
    pub fn new(
        user_totp_secret: S,
        authority_by_client_key: T,
        update_user_authority: U,
        select_user_authority: V,
    ) -> Self {
        Self {
            user_totp_secret,
            authority_by_client_key,
            update_user_authority,
            select_user_authority,
        }
    }
}

#[async_trait]
impl<'a, S, T, U, V> Service<&'a UpdatePasswordParams> for UpdatePasswordUseCase<S, T, U, V>
where
    S: SelectTOTPSecrețByUserIdQuery,
    T: SelectAuthorityByClientKeyQuery,
    U: UpdateUserAuthorityQuery,
    V: SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQuery,
{
    type Response = UpdatePasswordResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "update_password_usecase", skip(self))]
    async fn call(&self, params: &'a UpdatePasswordParams) -> Result<Self::Response, Self::Error> {
        // check password match
        if &params.password != &params.password_conf {
            return Err("password and password confirmation do not match".into());
        }

        // get authority params using provided client key
        let authority_res = self
            .authority_by_client_key
            .call(&FindAuthorityByClientKey {
                client_key: params.client_key,
            })
            .await;

        let Ok(Some(authority)) = authority_res else {
            return Err("Failed to find authority by client key".into());
        };

        // todo - get user authority by username & authority id
        let user_authority_res = self
            .select_user_authority
            .call(
                &SelectUserAuthoritiesByAuthorityIdAndUserIdentifierQueryParams {
                    authority_id: authority.id,
                    user_identifier: params.username.clone(),
                },
            )
            .await;

        let Ok(user_authority) = user_authority_res else {
            return Err("Failed to find user by username".into());
        };

        let secret_by_user_id = self
            .user_totp_secret
            .call(&FindTOTPSecretByUserId {
                user_id: user_authority.user_id,
            })
            .await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "time is before 1970")?;

        let valid = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(600)
            .timestamp(now.as_secs() as i64)
            .finalize()
            .unwrap()
            .is_valid(&params.code);

        info!("code validated. Result: {}", valid);

        if !valid {
            return Err("invalid code".into());
        }

        let authority_params: AuthorityParams = authority.params.try_into()?;

        let password_salt = authority_params.password_salt;

        // TODO: Salt & Pepper new password
        let password_pepper = std::env::var("OXIDAUTH_USERNAME_PASSWORD_PEPPER")?;

        let password = raw_password_hash(&params.password, &password_salt, &password_pepper);

        let password_hash = hash_password(password).map_err(|err| err.to_string())?;

        // TODO: Set new password on user
        let new_password_params = UserAuthorityParams { password_hash };

        let new_password_value = serde_json::to_value(new_password_params)?;

        let user_authority = UpdateUserAuthority {
            user_id: user_authority.user_id,
            authority_id: authority.id,
            params: JsonValue::new(new_password_value),
        };

        let res = self
            .update_user_authority
            .call(&user_authority)
            .await;

        Ok(UpdatePasswordResponse {
            success: res.is_ok(),
        })
    }
}
