use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    settings::{fetch_setting::FetchSettingParams, Setting},
};

use crate::Database;

use super::PgSetting;

#[async_trait]
impl<'a> Service<&'a FetchSettingParams> for Database {
    type Response = Option<Setting>;
    type Error = BoxedError;

    #[tracing::instrument(name = "select_setting_by_key_query", skip(self))]
    async fn call(
        &self,
        params: &'a FetchSettingParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgSetting>(include_str!(
            "./select_setting_by_key.sql"
        ))
        .bind(&params.key)
        .fetch_optional(&self.pool)
        .await?;

        let setting = result.map(Into::into);

        Ok(setting)
    }
}
