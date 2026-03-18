mod fetch_setting;
mod save_setting;

use fetch_setting::FetchSettingTrait;
use save_setting::SaveSettingTrait;

#[cfg(feature = "mock")]
use super::mock::ClientMock;
use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};

pub trait SettingsTrait: SaveSettingTrait + FetchSettingTrait {}

impl SettingsTrait for Client {
}

#[cfg(feature = "mock")]
impl SettingsTrait for ClientMock {
}
