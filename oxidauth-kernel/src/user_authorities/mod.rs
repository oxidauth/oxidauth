pub mod create_user_authority;
pub mod delete_user_authority;
pub mod find_user_authority_by_user_id_and_authority_id;
pub mod list_user_authorities_by_user_id;
pub mod update_user_authority;

use std::fmt;

use crate::{authorities::Authority, dev_prelude::*};

#[derive(Serialize, Deserialize)]
pub struct UserAuthority {
    pub user_id: Uuid,
    pub authority_id: Uuid,
    pub user_identifier: String,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Debug for UserAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserAuthority")
            .field("user_id", &self.user_id)
            .field(
                "authority_id",
                &self.authority_id,
            )
            .field(
                "user_identifier",
                &self.user_identifier,
            )
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAuthorityWithAuthority {
    pub user_authority: UserAuthority,
    pub authority: Authority,
}
