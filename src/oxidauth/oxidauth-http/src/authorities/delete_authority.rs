use oxidauth_kernel::authorities::delete_authority::Authority;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAuthorityRes {
    pub authority: Authority,
}
