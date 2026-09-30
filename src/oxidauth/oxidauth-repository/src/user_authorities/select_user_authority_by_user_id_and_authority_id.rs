pub use oxidauth_kernel::user_authorities::{
    UserAuthorityWithAuthority,
    find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityId,
};

use crate::prelude::*;

#[async_trait]
pub trait SelectUserAuthorityByUserIdAndAuthorityIdQuery: Send + Sync + 'static {
    async fn select_user_authority_by_user_id_and_authority_id(
        &self,
        params: &FindUserAuthorityByUserIdAndAuthorityId,
    ) -> Result<UserAuthorityWithAuthority, BoxedError>;
}
