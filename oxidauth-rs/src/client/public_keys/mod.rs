pub mod create_public_key;
pub mod delete_public_key;
pub mod find_public_key_by_id;
pub mod list_all_public_keys;

#[cfg(feature = "mock")]
use super::mock::ClientMock;
use super::{
    Client,
    Resource,
    fmt,
    handle_response,
};
use crate::public_keys::{
    create_public_key::CreatePublicKeyTrait,
    delete_public_key::DeletePublicKeyTrait,
    find_public_key_by_id::FindPublicKeyByIdTrait,
    list_all_public_keys::ListAllPublicKeysTrait,
};

pub trait PublicKeysTrait:
    ListAllPublicKeysTrait
    + FindPublicKeyByIdTrait
    + DeletePublicKeyTrait
    + CreatePublicKeyTrait
{
}

impl PublicKeysTrait for Client {
}

#[cfg(feature = "mock")]
impl PublicKeysTrait for ClientMock {
}
