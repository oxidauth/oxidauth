pub mod create_authority;
pub mod delete_authority;
pub mod find_authority_by_id;
pub mod find_authority_by_strategy;
pub mod list_all_authorities;
pub mod update_authority;

pub use oxidauth_kernel::authorities::NbfOffset;
pub use oxidauth_kernel::authorities::create_authority::*;

#[cfg(feature = "mock")]
use super::mock::ClientMock;
use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};
pub use crate::authorities::{
    create_authority::CreateAuthorityTrait,
    delete_authority::DeleteAuthorityTrait,
    find_authority_by_id::FindAuthorityByIdTrait,
    find_authority_by_strategy::FindAuthorityByStrategyTrait,
    list_all_authorities::ListAllAuthoritiesTrait,
    update_authority::UpdateAuthorityTrait,
};

pub trait AuthoritiesTrait:
    CreateAuthorityTrait
    + DeleteAuthorityTrait
    + FindAuthorityByIdTrait
    + FindAuthorityByStrategyTrait
    + ListAllAuthoritiesTrait
    + UpdateAuthorityTrait
{
}

impl AuthoritiesTrait for Client {
}

#[cfg(feature = "mock")]
impl AuthoritiesTrait for ClientMock {
}
