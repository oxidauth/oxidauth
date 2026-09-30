use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, settings::fetch_setting::FetchSettingParams};
use oxidauth_repository::settings::select_setting_by_key::*;

use super::*;

#[async_trait]
impl SelectSettingByKey for PgSettingRepository {
    #[tracing::instrument(name = "select_setting_by_key_query", skip(self))]
    async fn select_setting_by_key(
        &self,
        params: &FetchSettingParams,
    ) -> Result<Option<Setting>, BoxedError> {
        let result = sqlx::query_as::<_, PgSetting>(include_str!("./select_setting_by_key.sql"))
            .bind(&params.key)
            .fetch_optional(&self.db.read_pool())
            .await?;

        let setting = result.map(Into::into);

        Ok(setting)
    }
}

#[cfg(test)]
mod tests {

    use oxidauth_kernel::settings::save_setting::SaveSettingParams;
    use oxidauth_repository::settings::upsert_setting::*;
    use serde_json::json;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::Database;

    fn repo(pool: &PgPool) -> PgSettingRepository {
        PgSettingRepository::new(Database::from_pool(pool.clone()))
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_find_a_setting_by_key(pool: PgPool) {
        let key = format!("feature-flag-{}", Uuid::new_v4());
        repo(&pool)
            .save_setting(&SaveSettingParams {
                key: key.clone(),
                value: json!({ "enabled": true }),
            })
            .await
            .expect("upsert should succeed");

        let found = repo(&pool)
            .select_setting_by_key(&FetchSettingParams { key: key.clone() })
            .await
            .expect("query should succeed");

        let setting = found.expect("the setting should be present");
        assert_eq!(setting.key, key);
        assert_eq!(setting.value, json!({ "enabled": true }));
        assert!(setting.created_at.timestamp() > 0);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_return_none_for_a_missing_key(pool: PgPool) {
        // pinned: the repository uses fetch_optional and surfaces Ok(None) —
        // SettingNotFoundError is produced upstream in the services layer
        let found = repo(&pool)
            .select_setting_by_key(&FetchSettingParams {
                key: format!("no-such-setting-{}", Uuid::new_v4()),
            })
            .await
            .expect("a missing key must not error at this layer");

        assert!(
            found.is_none(),
            "expected None for a missing key, got: {found:?}"
        );
    }
}
