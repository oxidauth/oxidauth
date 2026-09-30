use oxidauth_kernel::users::{User, find_users_by_ids::FindUsersByIds};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type FindUsersByIdsReq = FindUsersByIds;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindUsersByIdsRes {
    pub users: Vec<User>,
    pub user_ids_not_found: Vec<Uuid>,
}
