pub use oxidauth_kernel::auth::oauth2::state::ConsumeOAuthState;

pub use crate::prelude::*;

#[async_trait]
pub trait ConsumeOAuthStateQuery: Send + Sync + 'static {
    /// Atomically deletes and consumes the nonce iff it exists, was minted
    /// for `client_key`, and has not expired. `true` = the state was live and
    /// is now spent; `false` = unknown, expired, spent, or bound to another
    /// authority (the distinction must not leak to callers, OXA-000010).
    async fn consume_oauth_state(&self, params: &ConsumeOAuthState) -> Result<bool, BoxedError>;
}
