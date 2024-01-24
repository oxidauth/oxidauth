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
