use async_trait::async_trait;

use oxidauth_kernel::{
    auth::username_password::forgot_password::{ForgotPasswordInfo, ForgotPasswordUseCaseRes},
    authorities::create_authority::*,
    error::BoxedError,
};
use oxidauth_repository::users::select_user_by_id_query::SelectUserByIdQuery;

pub struct ForgotPasswordUseCase<U>
where
    U: SelectUserByIdQuery,
{
    users: U,
}

impl<U> ForgotPasswordUseCase<U>
where
    U: SelectUserByIdQuery,
{
    pub fn new(users: U) -> Self {
        Self { users }
    }
}

#[async_trait]
impl<'a, U> Service<&'a mut ForgotPasswordInfo> for ForgotPasswordUseCase<U>
where
    U: SelectUserByIdQuery,
{
    type Response = ForgotPasswordUseCaseRes;
    type Error = BoxedError;

    #[tracing::instrument(name = "forgot_password_usecase", skip(self))]
    async fn call(&self, req: &'a mut ForgotPasswordInfo) -> Result<Self::Response, Self::Error> {
        // TODO: Delete existing refresh tokens by user id

        // TODO: GENERATE TOTP CODE
        // let code = TOTPBuilder::new()
        //     .ascii_key(&secret_by_user_id.secret)
        //     .period(totp_ttl.as_secs() as u32)
        //     .timestamp(now.as_secs() as i64)
        //     .finalize()
        //     .map_err(|err| format!("error generating totp: {:?}", err))?
        //     .generate();
    }
}
