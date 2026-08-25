pub mod create_public_key;
pub mod delete_public_key;
pub mod find_public_key_by_id;
pub mod list_all_public_keys;

use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "mock")]
use super::mock::ClientMock;

use super::{Client, Resource, fmt, handle_response};
pub use crate::public_keys::{
    create_public_key::CreatePublicKeyTrait, delete_public_key::DeletePublicKeyTrait,
    find_public_key_by_id::FindPublicKeyByIdTrait, list_all_public_keys::ListAllPublicKeysTrait,
};

pub trait PublicKeysTrait:
    ListAllPublicKeysTrait + FindPublicKeyByIdTrait + DeletePublicKeyTrait + CreatePublicKeyTrait
{
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllPublicKeysRes {
    pub public_keys: Vec<PublicKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKey {
    pub id: Uuid,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PublicKeysTrait for Client {}

#[cfg(feature = "mock")]
impl PublicKeysTrait for ClientMock {}
