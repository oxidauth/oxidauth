pub mod create_user_authority;
pub mod delete_user_authority;
pub mod find_user_authority_by_user_id_and_authority_id;
pub mod list_user_authorities_by_user_id;
pub mod update_user_authority;

use std::{error::Error, fmt};

use crate::{JsonValue, authorities::Authority, dev_prelude::*};

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

#[derive(Debug)]
pub struct UserAuthorityNotFoundError {
    pub authority_id: Uuid,
    pub user_identifier: String,
}

impl UserAuthorityNotFoundError {
    pub fn new(authority_id: Uuid, user_identifier: String) -> Box<Self> {
        Box::new(Self {
            authority_id,
            user_identifier,
        })
    }
}

impl fmt::Display for UserAuthorityNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "user authority not found: authority_id {} user_identifier {}",
            self.authority_id, self.user_identifier
        )
    }
}

impl Error for UserAuthorityNotFoundError {
}
