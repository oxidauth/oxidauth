//! Postgres database for the stack.
//!
//! The `database!` macro from the vendored `postgres` crate generates the
//! `Database` type (write + read pools), `DatabaseBuilder`, the `PingTrait`
//! impl, and re-exports `PgError`, `Ping`, `PingTrait`, and `mock` at this
//! crate root.
//!
//! Env vars:
//! - `DATABASE_URL` (required): write pool
//! - `READ_DATABASE_URL` (optional): read pool, falls back to `DATABASE_URL`
//! - `MIGRATIONS_ENABLED` (required): `"true"` runs migrations on `migrate()`;
//!   a missing var is a hard `PgError::MissingEnvVar` (fail-fast).
//!
//! Query implementations for the kernel entities live in the `auth`, `users`,
//! `roles`, etc. modules below as `impl Service<&Params> for Database`.

pub mod auth;
pub mod authorities;
pub mod invitations;
pub mod permissions;
pub mod private_keys;
pub mod public_keys;
pub mod refresh_tokens;
pub mod role_permission_grants;
pub mod role_role_grants;
pub mod roles;
pub mod settings;
pub mod totp_secrets;
pub mod user_authorities;
pub mod user_permission_grants;
pub mod user_role_grants;
pub mod users;

pub mod prelude;

const DATABASE_URL: &str = "DATABASE_URL";
const READ_DATABASE_URL: &str = "READ_DATABASE_URL";
const MIGRATIONS_ENABLED: &str = "MIGRATIONS_ENABLED";

pub const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

postgres::database!(DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED, MIGRATOR);
