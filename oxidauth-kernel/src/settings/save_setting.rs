use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dev_prelude::BoxedError;

use super::Setting;

#[async_trait]
pub trait SaveSettingTrait: Send + Sync + 'static {
    async fn save_setting(
        &self,
        params: &SaveSettingParams,
    ) -> Result<Setting, BoxedError>;
}

pub type SaveSettingService = Arc<dyn SaveSettingTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSettingParams {
    pub key: String,
    pub value: Value,
}
