use oxidauth_kernel::authorities::list_all_authorities::{Authority, ListAllAuthorities};
use serde::{Deserialize, Serialize};

pub type ListAllAuthoritiesReq = ListAllAuthorities;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllAuthoritiesRes {
    pub authorities: Vec<Authority>,
}
