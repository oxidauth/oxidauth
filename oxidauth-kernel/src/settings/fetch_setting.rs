use core::fmt;
use std::sync::Arc;

use serde::Deserialize;

use crate::dev_prelude::{BoxedError, Service};

use super::Setting;

pub type FetchSettingService = Arc<
    dyn for<'a> Service<
        &'a FetchSettingParams,
        Response = Setting,
        Error = BoxedError,
    >,
>;

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
