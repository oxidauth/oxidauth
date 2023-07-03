pub mod authenticate;
pub mod find_authority_by_client_id;
pub mod register;

pub use crate::user_authority::UserAuthority;
pub use authenticate::*;
pub use register::*;

use crate::dev_prelude::*;

#[derive(Debug)]
pub struct Authority {
    pub id: Uuid,
    pub name: String,
    pub client_key: Uuid,
    pub status: AuthorityStatus,
    pub strategy: AuthorityStrategy,
    pub settings: Value,
    pub params: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum AuthorityStatus {
    Enabled,
    Disabled,
}

#[derive(Debug)]
pub enum AuthorityStrategy {
    UsernamePassword,
    SingleUseToken,
}
