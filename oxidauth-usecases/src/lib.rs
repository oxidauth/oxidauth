pub mod auth;
pub mod authorities;
pub mod bootstrap;
pub mod dev_prelude;
pub mod invitations;
pub mod permissions;
pub mod public_keys;
pub mod refresh_tokens;
pub mod role_permission_grants;
pub mod role_role_grants;
pub mod roles;
pub mod settings;
pub mod totp;
pub mod totp_secrets;
pub mod user_authorities;
pub mod user_permission_grants;
pub mod user_role_grants;
pub mod users;

use rand::{distributions, thread_rng, Rng};

fn random_string() -> String {
    let s = thread_rng()
        .sample_iter(&distributions::Alphanumeric)
        .take(32)
        .collect::<Vec<_>>();

    String::from_utf8_lossy(&s).to_string()
}
