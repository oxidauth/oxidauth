use oxidauth_kernel::authorities::find_authority_by_strategy::{
    Authority,
    FindAuthorityByStrategy,
};
use serde::{Deserialize, Serialize};

pub type FindAuthorityByStrategyReq = FindAuthorityByStrategy;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByStrategyRes {
    pub authority: Authority,
}
