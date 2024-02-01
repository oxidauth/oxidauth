pub mod create_user_authority;
pub mod delete_user_authority;
pub mod find_user_authority_by_user_id_and_authority_id;
pub mod list_user_authorities_by_user_id;
pub mod update_user_authority;

use crate::{authorities::Authority, dev_prelude::*, JsonValue};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthorityWithAuthority {
    pub user_authority: UserAuthority,
    pub authority: Authority,
}
