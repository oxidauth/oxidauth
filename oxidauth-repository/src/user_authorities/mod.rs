pub mod delete_user_authority_by_id;
pub mod insert_user_authority;
pub mod query_user_authorities_by_authority_id_and_user_identifier;
pub mod query_user_authorities_by_user_id;
pub mod update_user_authority;

pub type UserAuthorityRow = oxidauth_kernel::authorities::UserAuthority;
