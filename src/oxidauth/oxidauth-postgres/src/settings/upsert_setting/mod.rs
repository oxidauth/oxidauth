use async_trait::async_trait;
use oxidauth_kernel::{error::BoxedError, settings::save_setting::SaveSettingParams};
use oxidauth_repository::settings::upsert_setting::*;

use super::*;

#[async_trait]
impl SaveSettingQuery for PgSettingRepository {
    #[tracing::instrument(name = "upsert_setting_query", skip(self))]
    async fn save_setting(&self, params: &SaveSettingParams) -> Result<Setting, BoxedError> {
        let result = sqlx::query_as::<_, PgSetting>(include_str!("./upsert_setting.sql"))
            .bind(&params.key)
            .bind(&params.value)
            .fetch_one(&self.db.write_pool())
            .await?;

        let setting = result.into();

        Ok(setting)
    }
}

#[cfg(test)]
mod tests {

    use serde_json::{Value, json};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::Database;

    fn repo(pool: &PgPool) -> PgSettingRepository {
        PgSettingRepository::new(Database::from_pool(pool.clone()))
    }

    async fn save(pool: &PgPool, key: &str, value: Value) -> Setting {
        repo(pool)
            .save_setting(&SaveSettingParams {
                key: key.to_owned(),
                value,
            })
            .await
            .expect("upsert should succeed")
    }

    async fn row_count(pool: &PgPool, key: &str) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM settings WHERE key = $1")
            .bind(key)
            .fetch_one(pool)
            .await
            .expect("count should run")
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_update_in_place_when_the_same_key_is_saved_twice(pool: PgPool) {
        // P0: plan-11's design — ON CONFLICT (key) DO UPDATE, never two rows
        let key = format!("bootstrap-{}/repeat", Uuid::new_v4());

        let first = save(&pool, &key, json!({ "done": false })).await;
        let second = save(&pool, &key, json!({ "done": true, "at": "rotation" })).await;

        assert_eq!(second.key, key);
        assert_eq!(
            second.value,
            json!({ "done": true, "at": "rotation" }),
            "the second save must win"
        );
        assert_eq!(
            second.created_at, first.created_at,
            "created_at belongs to the insert"
        );

        assert_eq!(
            row_count(&pool, &key).await,
            1,
            "upsert must leave exactly one row"
        );

        let stored: Value = sqlx::query_scalar("SELECT value FROM settings WHERE key = $1")
            .bind(&key)
            .fetch_one(&pool)
            .await
            .expect("stored value");
        assert_eq!(stored, json!({ "done": true, "at": "rotation" }));
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn it_should_keep_distinct_keys_independent(pool: PgPool) {
        let salt = Uuid::new_v4();
        let key_a = format!("setting-a-{salt}");
        let key_b = format!("setting-b-{salt}");

        save(&pool, &key_a, json!({ "nested": [1, 2, { "deep": true }] })).await;
        save(&pool, &key_b, json!("scalar string")).await;

        let (value_a, value_b): (Value, Value) = sqlx::query_as(
            "SELECT (SELECT value FROM settings WHERE key = $1) AS a, \
                    (SELECT value FROM settings WHERE key = $2) AS b",
        )
        .bind(&key_a)
        .bind(&key_b)
        .fetch_one(&pool)
        .await
        .expect("both values");

        assert_eq!(value_a, json!({ "nested": [1, 2, { "deep": true }] }));
        assert_eq!(
            value_b,
            json!("scalar string"),
            "jsonb keeps scalar values too"
        );
        assert_eq!(row_count(&pool, &key_a).await, 1);
        assert_eq!(row_count(&pool, &key_b).await, 1);
    }
}
