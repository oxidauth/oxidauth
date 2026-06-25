mod fetch_setting;
mod save_setting;

#[cfg(feature = "mock")]
use super::mock::ClientMock;

use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};

pub use crate::settings::{
    fetch_setting::FetchSettingTrait,
    save_setting::SaveSettingTrait,
};

pub trait SettingsTrait: SaveSettingTrait + FetchSettingTrait {}

impl SettingsTrait for Client {
}

#[cfg(feature = "mock")]
impl SettingsTrait for ClientMock {
}
