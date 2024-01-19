use crate::{dev_prelude::*, users::user_create::UserCreate};

#[async_trait]
pub trait RegisterService<E>: Send + Sync + 'static
where
    E: RegisterParamsExtractor,
{
    async fn register(&self, params: E) -> Result<(), RegisterError>;
}

pub trait RegisterService2<E>:
    Service<E, Response = (), Error = RegisterError>
{
    // fn register(
    //     &self,
    //     params: E,
    // ) -> impl Future<Output = Result<(), RegisterError>> + Send {
    //     self.call(params)
    // }
}

#[derive(Debug)]
pub struct RegisterError {}

#[async_trait]
pub trait RegisterParamsExtractor: Send + Sync + 'static {
    fn client_id(&self) -> Result<Uuid, RegisterParamsExtractorError>;
    fn user_identifier(&self) -> Result<String, RegisterParamsExtractorError>;
    fn user_authority_params(
        &self,
    ) -> Result<Value, RegisterParamsExtractorError>;
    fn user_create(&self) -> Result<UserCreate, RegisterParamsExtractorError>;
}

#[derive(Debug)]
pub struct RegisterParamsExtractorError {}
