# OXA-000050 — Duplicate registration surfaces raw Postgres unique-violation text to anonymous clients

**Original ID:** SRV-14 · **Severity:** P3 · **Type:** bug · **Status:** implemented 2026-09-30 (coder+reviewer approved; UserAlreadyExistsError struct (Debug-name wire mapping need) w/ sanitized Display + no source-chain, 23505→typed map at insert_user — review minor fixed same-day: narrowed to users_username_key constraint, users_pkey id-clashes stay raw + regression test, pg 15/15 live; hurl register/users flipped + run live 2x; RED leak/GREEN zero-leak captured live; registrar pin retired to intentional-layering note; user_authorities 23505 noted as follow-on)
**Review tier:** Tier 4 (moderate — multi-file recipes, pinned-test flips) · reviewed 2026-09-30 · SCHEDULED (owner approved; scope = `UserAlreadyExistsError` in kernel/users (Display sanitized, username Debug-only) + SQLSTATE 23505 map at `insert_user` boundary (RowNotFound precedent) + flip mocks/asserts/**both hurl contracts** (`duplicate key` → `UserAlreadyExistsError`); registrar pass-through pin kept as intentional-layering note; status stays 400 — response-class/enumeration question stays OXA-000005's (deferred), deliberately not bundled; 3 files ~40 lines; only live anonymous-facing leak in tier)


## Locations

All paths relative to `src/oxidauth/` unless noted. Verified against the working tree 2026-09-29.

- **Register-line drift (verified):** the cited `oxidauth-services/src/auth/strategies/username_password/registrar.rs:284` is a `BUG(pinned)` comment inside the test `does_not_reject_a_duplicate_username` (`:282-301`, marker text "audit B1.3 expects duplicate-user rejection here"). The defect itself is the code the test describes, not the marker line.
- `oxidauth-services/src/auth/strategies/username_password/registrar.rs:25-64` — `UsernamePassword::register`: parses params, rejects confirmation mismatch (`:42-44`), hashes the password (`:48-51`), builds `CreateUser` + `CreateUserAuthority` (`:57-61`). **No uniqueness check anywhere.** The registrar struct holds only `authority_id`, `params`, `password_pepper` (`new()` at `:67-80`) — it has no repository handle and the `Registrar` trait contract (`oxidauth-kernel/src/auth/mod.rs:21-28`, returns `(CreateUser, CreateUserAuthority)`) gives registrars no DB access at all.
- `oxidauth-services/src/auth/strategies/oauth2/registrar.rs:19-36` — the sibling OAuth2 registrar is identical in shape: params → `CreateUser`, no uniqueness check (same trait limitation).
- `oxidauth-services/src/auth/register.rs:90-98` — `RegisterUseCase::register` is where duplicates actually die: `registrar.register(...)` (`:90-92`) then `self.users.call(&user)` (`:94` — this is `InsertUserQuery`; the 23505 fires here) then `self.user_authorities.call(...)` (`:96-98`). Each step propagates via bare `?`; no error classification, and no transaction spanning the inserts.
- `oxidauth-postgres/src/users/insert_user/mod.rs:16-49` — the persistence endpoint: `fetch_one(...).await?` (`:41`), so the raw `sqlx::Error::Database` unique violation is boxed as the `BoxedError` verbatim.
- `oxidauth-postgres/migrations/20221019185709_create_users.sql:5` — `username VARCHAR(64) NOT NULL UNIQUE` — the constraint (default-named `users_username_key`) that actually rejects duplicates.
- `oxidauth-api/src/server/api/v1/auth/register.rs:36-43` — the public route handler: any `Err` becomes `Response::bad_request().error(err.into_error())`. No auth extraction on the handler (`:12-15`) — registration is unauthenticated by design.
- `oxidauth-kernel/src/error.rs:32-44` — `IntoOxidAuthError for BoxedError`: serializes `display: format!("{}", err)` and `debug: format!("{:?}", err)` verbatim into the response body.
- `hurl/tests/register.hurl:56-69` — HTTP suite pins the current behavior: duplicate register → 400 with `$.errors[0].debug` containing `"duplicate key"`. `hurl/tests/users.hurl:71-74` pins the same raw text for `POST /api/v1/users` (same `insert_user` path).
- Pinned test asserting raw propagation: `oxidauth-services/src/auth/register.rs:691-726` `duplicate_username_propagates_without_writing_a_user_authority` (`:715` asserts `err.to_string()` contains `"duplicate key value violates unique constraint"`), driven by mock `:357-359` which returns the raw constraint string as an `Err`.
- Repository-level pin: `oxidauth-postgres/src/users/insert_user/mod.rs:127-156` `it_should_reject_duplicate_username` asserts the Debug string contains `"duplicate key"` and `"users_username_key"` (unmarked — plain test, not `BUG(pinned)`).
- Existing precedent for mapping at this boundary: `oxidauth-postgres/src/user_authorities/select_user_authorities_by_authority_id_and_user_identifier/mod.rs:32` already converts `sqlx::Error::RowNotFound` into a domain-shaped message `"user authority not found: .."`.
- Error-type convention to follow: `oxidauth-kernel/src/users/mod.rs:153-183` (`UserNotFoundError`: enum + `Box<Self>` constructors + `Display`); cf. `SettingNotFoundError`, `PermissionNotFoundError`.

## Problem

A duplicate username at registration is never recognized as a domain condition. It passes through the registrar untouched, dies at the Postgres `users_username_key` unique index, and the resulting raw `sqlx` error is serialized word-for-word into the 400 body of an **unauthenticated** endpoint. Concretely, `POST /api/v1/auth/register` with an existing username returns (per `register.hurl:66-69` and the `into_error` pass-through) an envelope whose `display` is sqlx's `"error returned from database: duplicate key value violates unique constraint \"users_username_key\""` and whose `debug` is sqlx's derived Debug of `PgDatabaseError` — exposing SQLSTATE `23505`, the internal constraint name, and Postgres's message text. Whether the DETAIL line naming the *conflicting value* (`Key (username)=(x) already exists.`) is also present in the Debug render is standard Postgres behavior but was not verified against a live server here `[INFERENCE]`.

There is no typed error to match on anywhere between the database and the client: production code inspects SQLSTATE nowhere (workspace grep for `23505`/`.code()` outside test modules returns nothing), so no caller — HTTP handler, bootstrap, OAuth2 auto-register — can distinguish "username taken" from a connection failure. Every one of them returns the same raw text in a 400.

## Analysis

**Mechanism.** `RegisterUseCase::register` (`services/auth/register.rs:81-163`) is a linear `?`-chain: authority lookup → `registrar.register` → `insert_user` (`:94`) → `insert_user_authority` (`:96-98`) → permission tree → private key → JWT → refresh token. The `Registrar` strategy layer is deliberately pure (params in, DTOs out), so "duplicate username" is only *observable* at the repository call — and that call maps nothing. `into_error` (`kernel/error.rs:32-44`) is a deliberate Debug/Display verbatim envelope (its own tests pin that hurl asserts against `$.errors[0].debug`), so whatever string the deepest driver produced reaches the wire.

**Why :284 can't be "fixed" where the marker sits.** The pinned test's audit expectation ("duplicate-user rejection here") is unachievable inside the registrar without breaking the `Registrar` trait contract (no DB access) and reintroducing check-then-insert — a TOCTOU race two concurrent registrations always lose anyway. The correct invariant is: *the unique index is the arbiter; the persistence boundary translates its verdict into a domain error.* The postgres crate is the only crate depending on `sqlx` (`oxidauth-postgres/Cargo.toml:23`; services/repository/kernel do not), so the downcast on `sqlx::Error::Database` must live there — exactly where `RowNotFound` is already translated (`select_user_authorities.../mod.rs:32`).

**The same-flow second leak.** `register.rs:96-98` can also raise 23505 against `user_authorities` (`UNIQUE (user_identifier, authority_id)` + composite PK, pinned at `oxidauth-postgres/src/user_authorities/insert_user_authority/mod.rs:116,158`) — reachable because the `?`-chain is **not transactional**: any failure after `insert_user` (`:94`) leaves an orphan user row, so a later legitimate retry with the same username can fail with a raw duplicate error for a half-created account. The transaction gap itself is out of scope here, but the typed error makes the state diagnosable instead of opaque.

**Enumeration angle.** The duplicate-vs-success split is a username-existence oracle — but it exists *de facto* today regardless of message text (200 means "I just created an account"), and the authenticate path already leaks existence via `"user authority not found: <identifier>"` (`oxidauth-postgres/src/user_authorities/select_user_authorities_by_authority_id_and_user_identifier/mod.rs:32`, surfaced through `authenticate_or_register.rs:198-205`'s string match). This is the same class as OXA-000005's 200-vs-400 oracle finding, and OXA-000005's prescription applies here: keep the true error in `tracing`, sanitize the wire copy. Blind-equalizing register responses (always generic "registration could not be completed") would push legitimate users into unresolvable states, so the recommended move is narrower: a typed domain error with sanitized user-facing copy that stops exposing SQLSTATE/constraint internals; eliminating the existence oracle itself is a product decision, not a P3 bugfix.

**Adjacent paths that inherit the fix or the bug.** `POST /api/v1/users` (create user) and the OAuth2 auto-register flow (`authenticate_or_register.rs:206-224`, username = profile email, raw `?` at `:214-218`) share `InsertUserQuery`, so mapping at `insert_user` heals them simultaneously — including the callback body where OXA-000037 (200-on-error) governs the status code. Bootstrap (`oxidauth-services/src/bootstrap/mod.rs:478-480`) likewise gets a typed error to log (its swallowing is OXA-000038's scope). `permissions`/`roles`/grant-table 23505s (`hurl/tests/permissions.hurl:48`, `roles.hurl:63`) stay raw — deliberately out of scope; this ticket maps the user-username constraint only.

**Who is affected.** Every deployment exposing `/api/v1/auth/register` (public by design). Any retry storm, double-submit, or OAuth2 login whose profile email collides with an existing username returns DB internals to end users and logs unclassifiable strings.

## Impact

- **Information disclosure (low-grade but free to fix):** anonymous callers learn SQLSTATE, index/constraint naming, and driver rendering from a public endpoint — exactly the schema detail `into_error` was supposed to be a policy layer over, not an amplifier of.
- **No programmatic handling:** SDKs and the web UI cannot branch on "username taken" to show a usable form error; today the only discriminator is substring-matching Postgres prose (`register.rs:715` and the hurl asserts are the codebase doing exactly that to itself).
- **Misleading failures:** half-registered accounts (orphan `users` row from a non-atomic flow) make *later* registrations fail with a raw duplicate for a username the user was never successfully given.

## Proposed resolution

**1. Typed domain error in the kernel** (`oxidauth-kernel/src/users/mod.rs`, following the `UserNotFoundError` pattern at `:153-183`):

```rust
pub enum UserAlreadyExistsError { Username(Username) }
impl UserAlreadyExistsError { pub fn username(username: &Username) -> Box<Self> { ... } }
impl fmt::Display for UserAlreadyExistsError {
    // sanitized copy — no SQLSTATE, no constraint name; do not echo the username
    // (Display reaches the HTTP body via into_error): "username is already taken"
}
```

Keep the offending username on the *struct* (for `Debug`/`tracing`), not in `Display`.

**2. Map SQLSTATE at the persistence boundary** (`oxidauth-postgres/src/users/insert_user/mod.rs`): on `Err(sqlx::Error::Database(db))` with `db.code().as_deref() == Some("23505")`, return `UserAlreadyExistsError::username(...)` (chain or drop the sqlx error as needed for logs); all other errors propagate unchanged. No check-then-insert anywhere — the unique index remains the sole arbiter, so the fix is TOCTOU-safe by construction. Leave `user_authorities` 23505 mapping as a follow-on note in-code comment (different message — "already registered with this authority" — and needs its own decision).

**3. Flip the pins** (each currently asserts the raw text and must be rewritten in the same commit):

- `oxidauth-services/src/auth/register.rs:357-359` mock → return `UserAlreadyExistsError::username(..)`; `:691-726` test assertion (`:715`) → downcast/`to_string` matches `"username is already taken"`; the "no dangling user authority" assertions (`:717-723`) stay.
- `oxidauth-postgres/src/users/insert_user/mod.rs:127-156` → assert the typed error (`:153`'s `users_username_key` containment assertion flips; keeping one SQLSTATE-level check inside this one test is fine).
- `hurl/tests/register.hurl:69` and `hurl/tests/users.hurl:74` → assert `$.errors[0].debug contains "UserAlreadyExistsError"` (matches the `SettingNotFoundError`/`PermissionNotFoundError` hurl convention and `into_error`'s Debug rendering).
- `oxidauth-services/.../username_password/registrar.rs:282-301` `does_not_reject_a_duplicate_username` (marker `:284-286`): **registrar-level behavior does not change** — duplicates still pass through a DB-less registrar. Rewrite the `BUG(pinned)` marker into an intentional-layering note (the OXA-000033 Option-B pattern): "uniqueness is arbitrated by the DB at the insert boundary; the registrar stays pure; see OXA-000050", and rename the test to e.g. `duplicate_detection_lives_at_the_insert_boundary` so the register item retires honestly instead of pretending the registrar will grow a check.

**4. Compat:** the error *class* of the response is unchanged (HTTP 400, `errors[0]` present, `success:false`) — only the `display`/`debug` strings change, and `oxidauth-rs`'s client envelope handling (`client/mod.rs` `handle_response`) is string-agnostic. No wire/schema change; no migration. Out of scope, noted for the record: response status semantics for "taken" (409) and closing the existence oracle belong with OXA-000005's blinding recommendations.

## Verification

- `cargo test -p oxidauth-postgres insert_user` — `it_should_reject_duplicate_username` now yields the typed error; non-constraint errors (e.g. NOT NULL) still propagate raw.
- `cargo test -p oxidauth-services auth::register` — dispatch tests green; `duplicate_username_propagates_without_writing_a_user_authority` asserts the typed message and still no `insert_user_authority`/`private_key` ops after the failure.
- `cargo test -p oxidauth-services registrar` — renamed registrar test still proves pass-through (duplicate returns `Ok` at the strategy layer).
- `cargo check --workspace` — no other callers matched; kernel type addition is additive.
- Live/integration: `./hurl.sh` (register.hurl + users.hurl flip to `UserAlreadyExistsError`); manual smoke — `POST /api/v1/auth/register` twice with the same username, assert the second body contains neither `23505`, `users_username_key`, nor `duplicate key`, and that the server log (`tracing`) still records the username; then `POST /api/v1/auth/authenticate` with the same credentials still succeeds (user really exists — proves the index, not a false positive).
