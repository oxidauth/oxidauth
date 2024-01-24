use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::dev_prelude::{BoxedError, Service};

use super::Setting;

pub type SaveSettingService = Arc<
    dyn for<'a> Service<
        &'a SaveSettingParams,
        Response = Setting,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct SaveSettingParams {
    pub key: String,
    pub value: Value,
}
