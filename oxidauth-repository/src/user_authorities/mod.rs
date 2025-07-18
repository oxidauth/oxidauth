pub mod delete_user_authority;
pub mod insert_user_authority;
pub mod select_user_authorities_by_authority_id_and_user_identifier;
pub mod select_user_authorities_by_user_id;
pub mod select_user_authority_by_user_id_and_authority_id;
pub mod update_user_authority;

pub type UserAuthorityRow = oxidauth_kernel::authorities::UserAuthority;
