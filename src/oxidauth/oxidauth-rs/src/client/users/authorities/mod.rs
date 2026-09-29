pub mod create_user_authority;
pub mod delete_user_authority;
pub mod find_user_authority_by_user_id_and_authority_id;
pub mod list_user_authorities_by_user_id;
pub mod update_user_authority;

use super::*;
#[cfg(feature = "mock")]
use crate::mock::ClientMock;
pub use crate::users::authorities::{
    create_user_authority::CreateUserAuthorityTrait,
    delete_user_authority::DeleteUserAuthorityTrait,
    find_user_authority_by_user_id_and_authority_id::FindUserAuthorityByUserIdAndAuthorityIdTrait,
    list_user_authorities_by_user_id::ListUserAuthoritiesByUserIdTrait,
    update_user_authority::UpdateUserAuthorityTrait,
};

pub trait UserAuthoritiesTrait:
    UpdateUserAuthorityTrait
    + ListUserAuthoritiesByUserIdTrait
    + FindUserAuthorityByUserIdAndAuthorityIdTrait
    + DeleteUserAuthorityTrait
    + CreateUserAuthorityTrait
{
}

impl UserAuthoritiesTrait for Client {
}

#[cfg(feature = "mock")]
impl UserAuthoritiesTrait for ClientMock {
}
