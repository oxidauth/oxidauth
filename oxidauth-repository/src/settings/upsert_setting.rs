use oxidauth_kernel::{
    error::BoxedError,
    service::Service,
    settings::{save_setting::SaveSettingParams, Setting},
};

pub trait SaveSettingQuery:
    for<'a> Service<&'a SaveSettingParams, Response = Setting, Error = BoxedError>
{
}

impl<T> SaveSettingQuery for T where
    T: for<'a> Service<
        &'a SaveSettingParams,
        Response = Setting,
        Error = BoxedError,
    >
{
}
