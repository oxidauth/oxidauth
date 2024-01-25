// use oxidauth_kernel::authorities::authenticate::*;
//
// use crate::dev_prelude::*;
//
// pub struct AuthenticateUseCase {}
//
// #[async_trait]
// impl<P> AuthenticateService<P> for AuthenticateUseCase
// where
//     P: AuthenticateParamsExtractor,
// {
//     async fn authenticate(&self, params: P) -> Result<(), AuthenticateError> {
//         let client_id = params
//             .client_id()
//             .await
//             .map_err(|_| AuthenticateError {})?;
//
//         todo!()
//     }
// }
