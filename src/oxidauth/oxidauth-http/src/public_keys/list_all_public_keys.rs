use oxidauth_kernel::public_keys::{PublicKey, list_all_public_keys::ListAllPublicKeys};
use serde::{Deserialize, Serialize};

pub type ListAllPublicKeysReq = ListAllPublicKeys;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllPublicKeysRes {
    pub public_keys: Vec<PublicKey>,
}
