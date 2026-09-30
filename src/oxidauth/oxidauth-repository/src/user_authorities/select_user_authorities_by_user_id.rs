pub use oxidauth_kernel::user_authorities::{
    UserAuthorityWithAuthority,
    list_user_authorities_by_user_id::ListUserAuthoritiesByUserId,
};

use crate::prelude::*;

#[async_trait]
pub trait SelectUserAuthoritiesByUserIdQuery: Send + Sync + 'static {
    async fn select_user_authorities_by_user_id(
        &self,
        params: &ListUserAuthoritiesByUserId,
    ) -> Result<Vec<UserAuthorityWithAuthority>, BoxedError>;
}
