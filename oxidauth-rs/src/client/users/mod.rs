pub mod authorities;
pub mod create_user;
pub mod delete_user;
pub mod find_user_by_id;
pub mod find_user_by_username;
pub mod find_users_by_ids;
pub mod list_all_users;
pub mod permissions;
pub mod roles;
pub mod update_user;

use super::{fmt, handle_response, Client, Resource};

pub use oxidauth_kernel::users::*;
