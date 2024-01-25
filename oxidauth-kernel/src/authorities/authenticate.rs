// use crate::dev_prelude::*;
//
// use super::*;
//
// #[async_trait]
// pub trait AuthenticateService<P>: Send + Sync + 'static
// where
//     P: AuthenticateParamsExtractor,
// {
//     async fn authenticate(&self, params: P) -> Result<(), AuthenticateError>;
// }
//
// #[derive(Debug)]
// pub struct AuthenticateError {}
//
// #[async_trait]
// pub trait AuthenticateParamsExtractor: Send + Sync + 'static {
//     async fn client_id(&self)
//         -> Result<Uuid, AuthenticateParamsExtractorError>;
//     async fn user_identifier(
//         &self,
//     ) -> Result<String, AuthenticateParamsExtractorError>;
//     async fn params(
//         &self,
//         authority: &Authority,
//         user_authority: &UserAuthority,
//     ) -> Result<Value, AuthenticateParamsExtractorError>;
// }
//
// #[derive(Debug)]
// pub struct AuthenticateParamsExtractorError {}
