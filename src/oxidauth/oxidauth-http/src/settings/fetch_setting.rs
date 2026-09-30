use oxidauth_kernel::settings::Setting;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchSettingReq {
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchSettingRes {
    pub setting: Setting,
}
