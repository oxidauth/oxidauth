use oxidauth_kernel::public_keys::PublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePublicKeyRes {
    pub public_key: PublicKey,
}
