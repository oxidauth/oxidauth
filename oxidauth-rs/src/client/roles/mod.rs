pub mod permissions;
#[allow(clippy::module_inception)]
pub mod roles;

mod create_role;
mod delete_role;
mod file_role_by_id;
mod list_all_roles;
mod update_role;

use super::{handle_response, Client, ClientError, ClientErrorKind, Resource};
