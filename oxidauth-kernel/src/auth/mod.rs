pub mod authenticate;
pub mod tree;

use async_trait::async_trait;
use serde_json::Value;

use crate::{authorities::UserAuthority, dev_prelude::BoxedError};

// #[async_trait]
// pub trait Registrar<Params>: Send + Sync + 'static {
//     async fn register(&self, params: Params);
// }
//
// #[async_trait]
// pub trait Authenticator<Params>: Send + Sync + 'static {
//     async fn authenticate(
//         &self,
//         params: Params,
//         user_authority: &UserAuthority,
//     );
// }

#[async_trait]
pub trait Registrar: Send + Sync + 'static {
    async fn register(&self, params: Value);
}

#[async_trait]
pub trait Authenticator: Send + Sync + 'static {
    async fn authenticate(&self, params: Value, user_authority: &UserAuthority);

    async fn user_identifier_from_request(
        &self,
        request: &Value,
    ) -> Result<String, BoxedError>;
}
