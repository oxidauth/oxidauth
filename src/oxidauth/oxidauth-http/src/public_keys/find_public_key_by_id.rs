use oxidauth_kernel::public_keys::{PublicKey, find_public_key_by_id::FindPublicKeyById};
use serde::{Deserialize, Serialize};

pub type FindPublicKeyByIdReq = FindPublicKeyById;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPublicKeyByIdRes {
    pub public_key: PublicKey,
}
