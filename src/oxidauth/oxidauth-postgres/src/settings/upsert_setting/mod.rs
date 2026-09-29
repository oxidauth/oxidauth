use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    settings::{save_setting::SaveSettingParams, Setting},
};

use super::PgSetting;
use crate::Database;

#[async_trait]
impl<'a> Service<&'a SaveSettingParams> for Database {
    type Response = Setting;
    type Error = BoxedError;

    #[tracing::instrument(name = "upsert_setting_query", skip(self))]
    async fn call(
        &self,
        params: &'a SaveSettingParams,
    ) -> Result<Self::Response, Self::Error> {
        let result = sqlx::query_as::<_, PgSetting>(include_str!(
            "./upsert_setting.sql"
        ))
        .bind(&params.key)
        .bind(&params.value)
        .fetch_one(&self.pool)
        .await?;

        let setting = result.into();

        Ok(setting)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[sqlx::test]
    async fn it_should_be_able_to_upsert_a_new_setting(_pool: PgPool) {
        // let db = Database::new(pool).expect("should be able to make a db");
    }

    #[sqlx::test]
    async fn it_should_be_able_to_upsert_an_existing_setting(_pool: PgPool) {
        // let db = Database::new(pool).expect("should be able to make a db");
    }
}
