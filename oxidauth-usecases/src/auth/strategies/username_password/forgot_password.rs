use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

use boringauth::oath::TOTPBuilder;
use oxidauth_kernel::{
    auth::username_password::forgot_password::{ForgotPasswordParams, ForgotPasswordResponse},
    error::BoxedError,
    refresh_tokens::delete_refresh_token_by_user_id::DeleteRefreshTokenByUserId,
    service::Service,
    totp_secrets::find_totp_secret_by_user_id::FindTOTPSecretByUserId,
};
use oxidauth_repository::{
    refresh_tokens::delete_refresh_token_by_user_id::DeleteRefreshTokenByUserIdQuery,
    totp_secrets::select_totp_secret_by_user_id::SelectTOTPSecrețByUserIdQuery,
};

pub struct ForgotPasswordUseCase<D, S>
where
    D: DeleteRefreshTokenByUserIdQuery,
    S: SelectTOTPSecrețByUserIdQuery,
{
    delete_refresh_tokens_by_user_id: D,
    user_totp_secret: S,
}

impl<D, S> ForgotPasswordUseCase<D, S>
where
    D: DeleteRefreshTokenByUserIdQuery,
    S: SelectTOTPSecrețByUserIdQuery,
{
    pub fn new(delete_refresh_tokens_by_user_id: D, user_totp_secret: S) -> Self {
        Self {
            delete_refresh_tokens_by_user_id,
            user_totp_secret,
        }
    }
}

#[async_trait]
impl<'a, D, S> Service<&'a ForgotPasswordParams> for ForgotPasswordUseCase<D, S>
where
    D: DeleteRefreshTokenByUserIdQuery,
    S: SelectTOTPSecrețByUserIdQuery,
{
    type Response = ForgotPasswordResponse;
    type Error = BoxedError;

    #[tracing::instrument(name = "forgot_password_usecase", skip(self))]
    async fn call(&self, params: &'a ForgotPasswordParams) -> Result<Self::Response, Self::Error> {
        // generate code
        let secret_by_user_id = self
            .user_totp_secret
            .call(&FindTOTPSecretByUserId {
                user_id: params.user_id,
            })
            .await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "time is before 1970")?
            .as_secs() as i64;

        let code = TOTPBuilder::new()
            .ascii_key(&secret_by_user_id.secret)
            .period(600)
            .timestamp(now)
            .finalize()
            .map_err(|err| format!("error generating totp: {:?}", err))?
            .generate();

        // delete refresh tokens by id
        self.delete_refresh_tokens_by_user_id
            .call(&DeleteRefreshTokenByUserId {
                user_id: params.user_id,
            })
            .await?;

        Ok(ForgotPasswordResponse { code })
    }
}
