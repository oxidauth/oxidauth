use oxidauth_kernel::public_keys::{PublicKey, delete_public_key::DeletePublicKey};
use serde::{Deserialize, Serialize};

pub type DeletePublicKeyReq = DeletePublicKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletePublicKeyRes {
    pub public_key: PublicKey,
}
