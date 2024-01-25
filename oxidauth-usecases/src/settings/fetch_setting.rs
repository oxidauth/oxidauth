use core::fmt;

use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    settings::{fetch_setting::FetchSettingParams, Setting},
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
impl<'a, T> Service<&'a FetchSettingParams> for FetchSettingUseCase<T>
where
    T: SelectSettingByKey,
{
    type Response = Setting;
    type Error = BoxedError;

    #[tracing::instrument(name = "fetch_setting_usecase", skip(self))]
    async fn call(
        &self,
        params: &'a FetchSettingParams,
    ) -> Result<Self::Response, Self::Error> {
        let setting = self
            .select_setting
            .call(params)
            .await?;

        match setting {
            Some(setting) => Ok(setting),
            None => Err(SettingNotFoundError::new(
                &params.key,
            )),
        }
    }
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
