pub mod create_user_authority;
pub mod delete_user_authority;
pub mod find_user_authority_by_authority_and_user_identifier;
pub mod find_user_authority_by_user_id_and_authority_id;
pub mod list_user_authorities_by_user_id;
pub mod update_user_authority;
pub mod user_authority_create;

use crate::{authorities::Authority, dev_prelude::*};

#[derive(Debug, Serialize)]
pub struct UserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserAuthorityWithAuthority {
    pub user_authority: UserAuthority,
    pub authority: Authority,
}
