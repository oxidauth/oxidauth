use super::*;

impl Client {
    pub async fn username_password_forgot_password<T>(
        &self,
        params: ForgotPasswordParams,
    ) -> Result<Response<ForgotPasswordResponse>, String>
    where
        T: Into<ForgotPasswordParams> + fmt::Debug,
    {
        let result = self
            .post("/auth/username_password/forgot_password", params)
            .await;

        let Ok(resp) = result else {
            return Err("username_password_forgot_password call failed".to_string());
        };

        Ok(resp)
    }
}
