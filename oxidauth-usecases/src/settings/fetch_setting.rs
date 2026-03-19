use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    settings::{
        fetch_setting::{FetchSettingParams, FetchSettingTrait, SettingNotFoundError},
        Setting,
    },
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
impl<T> FetchSettingTrait for FetchSettingUseCase<T>
where
    T: SelectSettingByKey,
{
    #[tracing::instrument(name = "fetch_setting_usecase", skip(self))]
    async fn fetch_setting(
        &self,
        params: &FetchSettingParams,
    ) -> Result<Setting, BoxedError> {
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
