use oxidauth_kernel::authorities::find_authority_by_id::Authority;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByIdRes {
    pub authority: Authority,
}
