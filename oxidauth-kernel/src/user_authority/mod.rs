pub mod find_user_authority_by_authority_and_user_identifier;
pub mod user_authority_create;

use crate::dev_prelude::*;

#[derive(Debug)]
pub struct UserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
