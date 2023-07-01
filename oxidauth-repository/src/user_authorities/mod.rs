pub mod delete_user_authority_by_id;
pub mod insert_user_authority;
pub mod query_user_authorities_by_authority_id_and_user_identifier;
pub mod query_user_authorities_by_user_id;
pub mod update_user_authority;

pub type UserAuthorityRow = oxidauth_kernel::authorities::UserAuthority;

// #[derive(Debug)]
// pub struct UserAuthorityRow {
//     pub user_id: Uuid,
//     pub authority_id: Uuid,
//     pub user_identifier: String,
//     pub params: serde_json::Value,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: DateTime<Utc>,
// }

// @GEORGE - skipping postgres implementation for user_authorities for now
