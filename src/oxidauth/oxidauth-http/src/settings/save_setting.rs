use oxidauth_kernel::settings::{Setting, save_setting::SaveSettingParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSettingReq {
    pub setting: SaveSettingParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSettingRes {
    pub setting: Setting,
}
