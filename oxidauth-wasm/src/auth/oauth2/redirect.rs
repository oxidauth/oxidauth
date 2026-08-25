use super::*;

impl Client {
    pub async fn oauth2_redirect<T>(&self, params: T) -> Result<Response<Oauth2RedirectRes>, String>
    where
        T: Into<Oauth2RedirectParams> + fmt::Debug,
    {
        let params = params.into();

        let result = self
            .post("/auth/oauth2/redirect", params)
            .await;

        let Ok(resp) = result else {
            return Err("oauth2_redirect call failed".to_string());
        };

        Ok(resp)
    }
}
