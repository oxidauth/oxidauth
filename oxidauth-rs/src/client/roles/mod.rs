pub mod permissions;
#[allow(clippy::module_inception)]
pub mod roles;

pub mod create_role;
pub mod delete_role;
pub mod find_role_by_id;
pub mod find_role_by_name;
pub mod list_all_roles;
pub mod update_role;

use super::{fmt, handle_response, Client, Resource};
pub use oxidauth_kernel::roles::Role;
