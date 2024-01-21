pub mod tree;

use async_trait::async_trait;

use crate::authorities::UserAuthority;

#[async_trait]
pub trait Registrar<Params>: Send + Sync + 'static {
    async fn register(&self, params: Params);
}

#[async_trait]
pub trait Authenticator<Params>: Send + Sync + 'static {
    async fn authenticate(
        &self,
        params: Params,
        user_authority: &UserAuthority,
    );
}
