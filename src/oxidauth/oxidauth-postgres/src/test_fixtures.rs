//! Shared `#[cfg(test)]`-only seed helpers for the `oxidauth-postgres` repository
//! tests (audit A11.3).
//!
//! Isolation decision: `sqlx::test` gives every test its own freshly-migrated
//! database — created per test, dropped on success, no transaction rollback
//! (docs/TESTING.md §2) — so seeding STAYS per-test. The ~19 duplicated
//! single-INSERT seed bodies (~40 modules) live here. Import via `use crate::test_fixtures::...`.
//!
//! Fixtures that encode test-specific semantics on purpose — service write-path
//! seeds that pin the `Create*` request shape, auth-tree/cycle shapes, per-module
//! count helpers — intentionally stay inline in their own `mod tests`.

#![allow(dead_code, deprecated)]

use std::time::Duration;

use oxidauth_kernel::{
    JsonValue,
    authorities::{
        Authority,
        AuthoritySettings,
        AuthorityStatus,
        AuthorityStrategy,
        NbfOffset,
        TotpSettings,
        create_authority::CreateAuthority,
    },
    jwt::EntitlementsEncoding,
    users::create_user::CreateUser,
};
use oxidauth_repository::{
    authorities::insert_authority::InsertAuthorityQuery,
    users::insert_user::InsertUserQuery,
};
use sqlx::PgPool;

use crate::{authorities::PgAuthorityRepository, prelude::*, users::PgUserRepository};

/// The canonical authority settings every raw-SQL seed uses
/// (jwt 120s, no nbf, refresh 3600s, TOTP off, txt entitlements).
pub(crate) fn default_authority_settings() -> AuthoritySettings {
    AuthoritySettings {
        jwt_ttl: Duration::from_secs(120),
        jwt_nbf_offset: NbfOffset::Disabled,
        refresh_token_ttl: Duration::from_secs(3600),
        totp: TotpSettings::Disabled,
        entitlements_encoding: EntitlementsEncoding::Txt,
    }
}

/// Create an authority through the repository write path
/// (`CreateAuthority` service call); returns the created [`Authority`].
pub(crate) async fn create_authority(
    pool: &PgPool,
    name: &str,
    client_key: Uuid,
    status: AuthorityStatus,
    strategy: AuthorityStrategy,
    settings: AuthoritySettings,
    params: serde_json::Value,
) -> Authority {
    let params = CreateAuthority {
        name: name.to_owned(),
        client_key: Some(client_key),
        status: Some(status),
        strategy,
        settings,
        params: JsonValue::new(params),
    };

    PgAuthorityRepository::new(Database::from_pool(pool.clone()))
        .insert_authority(&params)
        .await
        .expect("seed authority should succeed")
}

/// Create a user through the repository write path (`CreateUser` service call,
/// all optional fields left to their column defaults); returns the new id.
pub(crate) async fn create_user(pool: &PgPool, username: &str) -> Uuid {
    let params = CreateUser {
        id: None,
        kind: None,
        status: None,
        username: username.to_owned(),
        email: None,
        first_name: None,
        last_name: None,
        profile: None,
    };

    PgUserRepository::new(Database::from_pool(pool.clone()))
        .insert_user(&params)
        .await
        .expect("seed user should succeed")
        .id
}

/// Assert the boxed error downcasts to a `sqlx::Error::Database` carrying the
/// expected SQLSTATE code (e.g. `"23505"`, `"23503"`).
pub(crate) fn assert_sql_state(err: BoxedError, expected: &str) {
    let sqlx_err = err
        .downcast_ref::<sqlx::Error>()
        .unwrap_or_else(|| panic!("expected a sqlx::Error, got: {err:?}"));
    match sqlx_err {
        sqlx::Error::Database(db_err) => {
            assert_eq!(
                db_err.code().as_deref(),
                Some(expected),
                "expected SQLSTATE {expected}, got: {db_err}"
            )
        },
        other => panic!("expected a database error, got: {other:?}"),
    }
}

/// Insert a `human`/`enabled` user with the generated username `user-{id}`.
pub(crate) async fn seed_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, kind, status, username) VALUES ($1, 'human', 'enabled', $2)",
    )
    .bind(id)
    .bind(format!("user-{id}"))
    .execute(pool)
    .await
    .expect("seed user");
    id
}

/// Insert a `human`/`enabled` user with the given username.
pub(crate) async fn seed_user_named(pool: &PgPool, username: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO users (kind, status, username) VALUES ('human', 'enabled', $1) \
         RETURNING id",
    )
    .bind(username)
    .fetch_one(pool)
    .await
    .expect("seed user")
}

/// Insert an `enabled` / `username_password` authority named `{label}-{id}`.
pub(crate) async fn seed_authority(pool: &PgPool, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO authorities (id, name, status, strategy) \
         VALUES ($1, $2, 'enabled', 'username_password')",
    )
    .bind(id)
    .bind(format!("{label}-{id}"))
    .execute(pool)
    .await
    .expect("seed authority");
    id
}

/// Like [`seed_authority`], but also stores [`default_authority_settings`] in the
/// `settings` jsonb column so joined read paths can decode them.
pub(crate) async fn seed_authority_with_settings(pool: &PgPool, label: &str) -> Uuid {
    let id = Uuid::new_v4();
    let settings = serde_json::to_value(default_authority_settings()).expect("settings serialize");
    sqlx::query(
        "INSERT INTO authorities (id, name, status, strategy, settings) \
         VALUES ($1, $2, 'enabled', 'username_password', $3)",
    )
    .bind(id)
    .bind(format!("{label}-{id}"))
    .bind(settings)
    .execute(pool)
    .await
    .expect("seed authority");
    id
}

pub(crate) async fn seed_role(pool: &PgPool, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO roles (id, name) VALUES ($1, $2)")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .expect("seed role");
    id
}

pub(crate) async fn seed_permission(
    pool: &PgPool,
    realm: &str,
    resource: &str,
    action: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO permissions (id, realm, resource, action) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(realm)
        .bind(resource)
        .bind(action)
        .execute(pool)
        .await
        .expect("seed permission");
    id
}

/// Insert a keypair row with the given public bytes and dummy private material.
pub(crate) async fn seed_public_key(pool: &PgPool, public_key: &[u8]) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO public_keys (id, public_key, private_key) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(public_key.to_vec())
        .bind(b"private-material".to_vec())
        .execute(pool)
        .await
        .expect("seed key");
    id
}

pub(crate) async fn seed_refresh_token(
    pool: &PgPool,
    user: Uuid,
    authority: Uuid,
    expires_at: DateTime<Utc>,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, authority_id, expires_at) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(user)
    .bind(authority)
    .bind(expires_at)
    .execute(pool)
    .await
    .expect("seed refresh token");
    id
}

/// [`seed_refresh_token`] with `expires_at = NOW()` evaluated by Postgres.
pub(crate) async fn seed_refresh_token_now(pool: &PgPool, user: Uuid, authority: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, authority_id, expires_at) \
         VALUES ($1, $2, $3, NOW())",
    )
    .bind(id)
    .bind(user)
    .bind(authority)
    .execute(pool)
    .await
    .expect("seed refresh token");
    id
}

pub(crate) async fn seed_role_permission_grant(pool: &PgPool, role: Uuid, permission: Uuid) {
    sqlx::query("INSERT INTO role_permission_grants (role_id, permission_id) VALUES ($1, $2)")
        .bind(role)
        .bind(permission)
        .execute(pool)
        .await
        .expect("seed grant");
}

pub(crate) async fn seed_role_role_grant(pool: &PgPool, parent: Uuid, child: Uuid) {
    sqlx::query("INSERT INTO role_role_grants (parent_id, child_id) VALUES ($1, $2)")
        .bind(parent)
        .bind(child)
        .execute(pool)
        .await
        .expect("seed edge");
}

pub(crate) async fn seed_user_permission_grant(pool: &PgPool, user: Uuid, permission: Uuid) {
    sqlx::query("INSERT INTO user_permission_grants (user_id, permission_id) VALUES ($1, $2)")
        .bind(user)
        .bind(permission)
        .execute(pool)
        .await
        .expect("seed grant");
}

pub(crate) async fn seed_user_authority(
    pool: &PgPool,
    user: Uuid,
    authority: Uuid,
    identifier: &str,
    params: serde_json::Value,
) {
    sqlx::query(
        "INSERT INTO user_authorities (user_id, authority_id, user_identifier, params) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user)
    .bind(authority)
    .bind(identifier)
    .bind(params)
    .execute(pool)
    .await
    .expect("seed user authority");
}

pub(crate) async fn seed_totp_secret(pool: &PgPool, user: Uuid, secret: &str) {
    sqlx::query("INSERT INTO totp_secrets (user_id, totp_secret) VALUES ($1, $2)")
        .bind(user)
        .bind(secret)
        .execute(pool)
        .await
        .expect("seed secret");
}
