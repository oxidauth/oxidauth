use super::*;

impl Client {
    pub async fn username_password_update_password<T>(
        &self,
        params: UpdatePasswordParams,
    ) -> Result<Response<UpdatePasswordResponse>, String>
    where
        T: Into<UpdatePasswordParams> + fmt::Debug,
    {
        let result = self
            .post("/auth/username_password/update_password", params)
            .await;

        let Ok(resp) = result else {
            return Err("username_password_update_password call failed".to_string());
        };

        Ok(resp)
    }
}
