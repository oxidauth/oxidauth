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

use rand::{Rng, distributions, thread_rng};

fn random_string() -> String {
    let s = thread_rng()
        .sample_iter(&distributions::Alphanumeric)
        .take(32)
        .collect::<Vec<_>>();

    String::from_utf8_lossy(&s).to_string()
}

#[cfg(test)]
/// Serializes every test that mutates the process environment (or exercises
/// product code that reads it). `std::env::set_var`/`remove_var` are `unsafe`
/// in edition 2024 because they race with concurrent `get_var` calls against
/// the shared environ block; tests acquire this lock around any such pair.
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) struct EnvGuard {
    key: &'static str,
    prev: Option<String>,
    _lock: std::sync::MutexGuard<'static, ()>,
}

#[cfg(test)]
impl EnvGuard {
    /// Sets `key` for the lifetime of the guard (restoring any previous
    /// value on drop) while holding `ENV_LOCK`.
    pub(crate) fn set(key: &'static str, value: &str) -> Self {
        let _lock = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let prev = std::env::var(key).ok();

        // SAFETY: every test in this binary that mutates or (via product
        // code) reads the process environment does so while holding
        // `ENV_LOCK`, so no `set_var` runs concurrently with a `get_var`.
        unsafe { std::env::set_var(key, value) };

        Self { key, prev, _lock }
    }

    /// Removes `key` for the lifetime of the guard (restoring any previous
    /// value on drop) while holding `ENV_LOCK`.
    pub(crate) fn unset(key: &'static str) -> Self {
        let _lock = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let prev = std::env::var(key).ok();

        if prev.is_some() {
            // SAFETY: as in `set` — `ENV_LOCK` serializes all env mutation.
            unsafe { std::env::remove_var(key) };
        }

        Self { key, prev, _lock }
    }
}

#[cfg(test)]
impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(prev) =>
            // SAFETY: still holding `ENV_LOCK` (field drop order: `_lock`
            // drops after this body runs).
            unsafe { std::env::set_var(self.key, prev) },
            None =>
            // SAFETY: as above.
            unsafe { std::env::remove_var(self.key) },
        }
    }
}
