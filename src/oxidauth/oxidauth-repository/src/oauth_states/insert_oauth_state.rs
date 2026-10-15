pub use oxidauth_kernel::auth::oauth2::state::CreateOAuthState;

pub use crate::prelude::*;

#[async_trait]
pub trait InsertOAuthStateQuery: Send + Sync + 'static {
    async fn insert_oauth_state(&self, params: &CreateOAuthState) -> Result<(), BoxedError>;
}
