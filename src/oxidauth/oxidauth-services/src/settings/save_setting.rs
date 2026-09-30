use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    settings::{
        Setting,
        save_setting::{SaveSettingParams, SaveSettingServiceTrait},
    },
};
use oxidauth_repository::settings::upsert_setting::SaveSettingQuery;

pub struct SaveSettingUseCase<T>
where
    T: SaveSettingQuery,
{
    save_setting: T,
}

impl<T> SaveSettingUseCase<T>
where
    T: SaveSettingQuery,
{
    pub fn new(save_setting: T) -> Self {
        Self { save_setting }
    }
}

#[async_trait]
impl<T> SaveSettingServiceTrait for SaveSettingUseCase<T>
where
    T: SaveSettingQuery,
{
    #[tracing::instrument(name = "SaveSettingUseCase::save_setting", skip(self))]
    async fn save_setting(&self, params: &SaveSettingParams) -> Result<Setting, BoxedError> {
        self.save_setting
            .save_setting(params)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use serde_json::{Value, json};

    use super::*;

    #[derive(Default)]
    struct Log {
        saves: Vec<(String, Value)>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn saves(log: &SharedLog) -> Vec<(String, Value)> {
        log.lock()
            .expect("log")
            .saves
            .clone()
    }

    struct MockSettings {
        log: SharedLog,
        fail: bool,
    }

    #[async_trait]
    impl SaveSettingQuery for MockSettings {
        async fn save_setting(&self, req: &SaveSettingParams) -> Result<Setting, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .saves
                .push((req.key.clone(), req.value.clone()));

            if self.fail {
                return Err("simulated upsert failure".into());
            }

            Ok(Setting {
                key: req.key.clone(),
                value: req.value.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }

    fn use_case(log: &SharedLog, fail: bool) -> SaveSettingUseCase<MockSettings> {
        SaveSettingUseCase::new(MockSettings {
            log: log.clone(),
            fail,
        })
    }

    // Pure upsert delegate: the service never reads the old value —
    // repeated saves are plain repeat upserts (the SQL layer keeps one
    // row per key, pinned in A10).
    #[tokio::test]
    async fn every_save_is_a_blind_upsert_of_key_and_value() {
        let log = Arc::new(Mutex::new(Log::default()));
        let use_case = use_case(&log, false);

        use_case
            .save_setting(&SaveSettingParams {
                key: "bootstrap".to_owned(),
                value: json!({ "bootstrapped": false }),
            })
            .await
            .expect("first save succeeds");

        let setting = use_case
            .save_setting(&SaveSettingParams {
                key: "bootstrap".to_owned(),
                value: json!({ "bootstrapped": true }),
            })
            .await
            .expect("second save succeeds");

        assert_eq!(
            saves(&log),
            vec![
                ("bootstrap".to_owned(), json!({ "bootstrapped": false })),
                ("bootstrap".to_owned(), json!({ "bootstrapped": true })),
            ],
            "no fetch-before-write: both values hit the upsert query as-is"
        );
        assert_eq!(setting.value, json!({ "bootstrapped": true }));
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true)
            .save_setting(&SaveSettingParams {
                key: "bootstrap".to_owned(),
                value: json!(null),
            })
            .await
            .expect_err("upsert failure propagates");

        assert_eq!(error.to_string(), "simulated upsert failure");
    }
}
