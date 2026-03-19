use core::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::dev_prelude::BoxedError;

use super::Setting;

#[async_trait]
pub trait FetchSettingTrait: Send + Sync + 'static {
    async fn fetch_setting(
        &self,
        params: &FetchSettingParams,
    ) -> Result<Setting, BoxedError>;
}

pub type FetchSettingService = Arc<dyn FetchSettingTrait>;

#[derive(Debug, Deserialize)]
pub struct FetchSettingParams {
    pub key: String,
}

#[derive(Debug)]
pub struct SettingNotFoundError {
    key: String,
}

impl SettingNotFoundError {
    pub fn new(key: &str) -> Box<Self> {
        Box::new(Self {
            key: key.to_owned(),
        })
    }
}

impl fmt::Display for SettingNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "no setting found with key: {}",
            self.key
        )
    }
}

impl std::error::Error for SettingNotFoundError {}
