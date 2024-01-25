use async_trait::async_trait;
use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    settings::{fetch_setting::FetchSettingParams, Setting},
};

#[async_trait]
pub trait SelectSettingByKey:
    for<'a> Service<
    &'a FetchSettingParams,
    Response = Option<Setting>,
    Error = BoxedError,
>
{
}

impl<T> SelectSettingByKey for T where
    T: for<'a> Service<
        &'a FetchSettingParams,
        Response = Option<Setting>,
        Error = BoxedError,
    >
{
}
