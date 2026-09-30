use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    settings::{
        Setting,
        fetch_setting::{FetchSettingParams, FetchSettingServiceTrait, SettingNotFoundError},
    },
};
use oxidauth_repository::settings::select_setting_by_key::SelectSettingByKey;

pub struct FetchSettingUseCase<T>
where
    T: SelectSettingByKey,
{
    select_setting: T,
}

impl<T> FetchSettingUseCase<T>
where
    T: SelectSettingByKey,
{
    pub fn new(select_setting: T) -> Self {
        Self { select_setting }
    }
}

#[async_trait]
impl<T> FetchSettingServiceTrait for FetchSettingUseCase<T>
where
    T: SelectSettingByKey,
{
    #[tracing::instrument(name = "FetchSettingUseCase::fetch_setting", skip(self))]
    async fn fetch_setting(&self, params: &FetchSettingParams) -> Result<Setting, BoxedError> {
        let setting = self
            .select_setting
            .select_setting_by_key(params)
            .await?;

        match setting {
            Some(setting) => Ok(setting),
            None => Err(SettingNotFoundError::new(&params.key)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::Utc;
    use serde_json::json;

    use super::*;

    fn stored_setting(key: &str) -> Setting {
        Setting {
            key: key.to_owned(),
            value: json!({ "bootstrapped": true }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[derive(Default)]
    struct Log {
        lookups: Vec<String>,
    }

    type SharedLog = Arc<Mutex<Log>>;

    fn lookups(log: &SharedLog) -> Vec<String> {
        log.lock()
            .expect("log")
            .lookups
            .clone()
    }

    struct MockSettings {
        log: SharedLog,
        missing: bool,
        fail: bool,
    }

    #[async_trait]
    impl SelectSettingByKey for MockSettings {
        async fn select_setting_by_key(
            &self,
            req: &FetchSettingParams,
        ) -> Result<Option<Setting>, BoxedError> {
            self.log
                .lock()
                .expect("log")
                .lookups
                .push(req.key.clone());

            if self.fail {
                return Err("simulated select failure".into());
            }

            if self.missing {
                return Ok(None);
            }

            Ok(Some(stored_setting("bootstrap")))
        }
    }

    fn use_case(log: &SharedLog, missing: bool, fail: bool) -> FetchSettingUseCase<MockSettings> {
        FetchSettingUseCase::new(MockSettings {
            log: log.clone(),
            missing,
            fail,
        })
    }

    #[tokio::test]
    async fn key_reaches_the_query_and_the_row_is_returned() {
        let log = Arc::new(Mutex::new(Log::default()));

        let setting = use_case(&log, false, false)
            .fetch_setting(&FetchSettingParams {
                key: "bootstrap".to_owned(),
            })
            .await
            .expect("fetch succeeds");

        assert_eq!(lookups(&log), vec!["bootstrap".to_owned()]);
        assert_eq!(setting.key, "bootstrap");
        assert_eq!(setting.value, json!({ "bootstrapped": true }));
    }

    #[tokio::test]
    async fn missing_key_becomes_a_setting_not_found_error() {
        // the repo answers `Ok(None)` (pinned in A10); the error is minted
        // here — bootstrap idempotency depends on distinguishing this
        // from a real query error
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, true, false)
            .fetch_setting(&FetchSettingParams {
                key: "bootstrap".to_owned(),
            })
            .await
            .expect_err("Ok(None) is mapped to a not-found error");

        assert_eq!(error.to_string(), "no setting found with key: bootstrap");
        // bootstrap's idempotency gate downcasts the TYPED error — a
        // Display-only pin would let a `format!`-String refactor break it silently
        assert!(
            error
                .downcast_ref::<SettingNotFoundError>()
                .is_some()
        );
        assert_eq!(lookups(&log), vec!["bootstrap".to_owned()]);
    }

    #[tokio::test]
    async fn repository_error_surfaces_unmapped_not_as_not_found() {
        let log = Arc::new(Mutex::new(Log::default()));

        let error = use_case(&log, false, true)
            .fetch_setting(&FetchSettingParams {
                key: "bootstrap".to_owned(),
            })
            .await
            .expect_err("select failure propagates verbatim");

        assert_eq!(error.to_string(), "simulated select failure");
        assert!(
            !error
                .to_string()
                .contains("not found")
        );
    }
}
