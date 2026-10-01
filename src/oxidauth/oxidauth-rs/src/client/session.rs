// Session persistence for the wasm (browser) build.
//
// A browser `Client` must survive page reloads: process memory dies with the
// tab, so the raw JWT and refresh token are mirrored in `LocalStorage` under
// the keys pinned by `crate::wasm` (`OXIDAUTH_JWT` / `OXIDAUTH_REFRESH_TOKEN`,
// shared with `wasm::State`). Every hook below is a no-op natively — there the
// in-memory state *is* the session.
//
// Storage is never staler than memory: `persist` runs on every token write
// (`auth`, `refresh`, `recover_jwt`) while the state lock is held, so
// `hydrate` (page load, and before a refresh-token exchange to adopt a pair a
// sibling tab rotated) may overwrite memory from storage unconditionally.
//
// The wasm storage arms cannot be exercised natively (gloo-storage panics
// through js-sys off-wasm) — the same convention as `crate::wasm`'s tests:
// the semantics are pinned there, and the `logout` contract by the native
// tests in `client/mod.rs`.

use super::State;

#[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
pub fn hydrate(_state: &mut State) {
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub fn hydrate(state: &mut State) {
    use gloo_storage::{LocalStorage, Storage as _};

    use crate::wasm::{JWT_KEY, REFRESH_TOKEN_KEY};

    if let Ok(raw_jwt) = LocalStorage::get::<String>(JWT_KEY) {
        state.raw_jwt = Some(raw_jwt);
    }

    if let Ok(token) = LocalStorage::get::<String>(REFRESH_TOKEN_KEY) {
        state.refresh_token = uuid::Uuid::parse_str(&token).ok();
    }
}

#[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
pub fn persist(_state: &State) {
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub fn persist(state: &State) {
    use gloo_storage::{LocalStorage, Storage as _};

    use crate::wasm::{JWT_KEY, REFRESH_TOKEN_KEY};

    let _ = LocalStorage::set(JWT_KEY, state.raw_jwt.as_deref());
    let _ = LocalStorage::set(
        REFRESH_TOKEN_KEY,
        state
            .refresh_token
            .map(|token| token.to_string())
            .as_deref(),
    );
}

#[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
pub fn clear(state: &mut State) {
    *state = State::default();
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub fn clear(state: &mut State) {
    use gloo_storage::{LocalStorage, Storage as _};

    use crate::wasm::{JWT_KEY, REFRESH_TOKEN_KEY};

    *state = State::default();

    LocalStorage::delete(JWT_KEY);
    LocalStorage::delete(REFRESH_TOKEN_KEY);
}
