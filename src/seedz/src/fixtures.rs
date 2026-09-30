//! Idempotent local-dev fixtures: a second demo authority (`local-dev`),
//! three demo users covering the grant combinations (role-only, role +
//! direct permission, direct-permission-only), and one sample setting.
//!
//! Plain SQL through the write pool (no repository layer): this is fixture
//! data, not domain logic. Idempotent via find-then-insert on each table's
//! natural key (name / username / realm+resource+action / PK). When a row
//! already exists — seeded by an earlier run, or created by hand / through
//! the API before the first seed — each helper RESOLVES the live row's id
//! and threads it into the dependent grant inserts, so same-named rows are
//! reused (never duplicated, never an FK abort mid-run); fresh inserts
//! carry deterministic `f000…` UUIDs (parkinglot seedz convention: `f0` =
//! seed data) and re-running converges on the same counts.
//!
//! Deliberately absent: signing keys, the `oxidauth:admin` role/user, and
//! the default authority — bootstrap owns those (see `crate` docs). The
//! fixture users get no credentials: passwords are argon2-hashed with the
//! authority salt + server pepper at registration, so seeding them here
//! would fake what only the register flow produces. Use them as
//! identity/grant fixtures (list, `can`, user-authority probes), or
//! register a DIFFERENT username through the `local-dev` client key for a
//! login-able user.

use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::{BoxedError, Seeder};

const AUTHORITY_LOCAL_DEV_ID: u8 = 1;
/// Pinned so local web probes / hurl can target the realm without querying
/// the DB first; unique against bootstrap's `OXIDAUTH_DEFAULT_CLIENT_KEY`.
const AUTHORITY_LOCAL_DEV_CLIENT_KEY: u8 = 2;

/// Seed UUID slots share one `u8` space in disjoint bands (parkinglot
/// convention) so no two seeded rows collide: authority 1..=2 (row id +
/// pinned client key), users 10..=19, permissions 30..=39, roles 40..=49.
fn seed_uuid(slot: u8) -> Uuid {
    Uuid::parse_str(&format!("f0000000-0000-4000-8000-0000000000{slot:02x}"))
        .expect("valid seed uuid")
}

/// Mirrors the shape bootstrap's `AuthoritySettings` serializes (see the
/// default authority row): externally-tagged `totp`, Duration as
/// `{secs, nanos}`, lowercase `entitlements_encoding`.
const AUTHORITY_SETTINGS: &str = r#"{"totp":"disabled","jwt_ttl":{"secs":120,"nanos":0},"jwt_nbf_offset":{"enabled":{"secs":10,"nanos":0}},"refresh_token_ttl":{"secs":172800,"nanos":0},"entitlements_encoding":"txt"}"#;
/// Fixed dev salt: deterministic rows are the point; this realm is local
/// only and its passwords never leave the dev box.
const AUTHORITY_PARAMS: &str = r#"{"password_salt":"seedz-local-dev-salt-0000000000"}"#;

#[derive(Clone, Copy)]
struct Permission {
    slot: u8,
    realm: &'static str,
    resource: &'static str,
    action: &'static str,
}

impl Permission {
    /// `realm:resource:action`, doubling as the resolved-ids map key.
    fn key(&self) -> String {
        format!("{}:{}:{}", self.realm, self.resource, self.action)
    }
}

const DEMO_DASHBOARD_VIEW: Permission = Permission {
    slot: 30,
    realm: "demo",
    resource: "dashboard",
    action: "view",
};
const DEMO_DASHBOARD_EDIT: Permission = Permission {
    slot: 31,
    realm: "demo",
    resource: "dashboard",
    action: "edit",
};
const DEMO_REPORTS_EXPORT: Permission = Permission {
    slot: 32,
    realm: "demo",
    resource: "reports",
    action: "export",
};
const DEMO_AUDIT_READ: Permission = Permission {
    slot: 33,
    realm: "demo",
    resource: "audit",
    action: "read",
};

#[derive(Clone, Copy)]
struct Role {
    slot: u8,
    name: &'static str,
    permissions: &'static [&'static Permission],
}

const ROLE_VIEWER: Role = Role {
    slot: 40,
    name: "seedz:viewer",
    permissions: &[&DEMO_DASHBOARD_VIEW],
};
const ROLE_EDITOR: Role = Role {
    slot: 41,
    name: "seedz:editor",
    permissions: &[&DEMO_DASHBOARD_VIEW, &DEMO_DASHBOARD_EDIT],
};

const ROLES: [&Role; 2] = [&ROLE_VIEWER, &ROLE_EDITOR];

#[derive(Clone, Copy)]
struct User {
    slot: u8,
    username: &'static str,
    email: &'static str,
    first_name: &'static str,
    last_name: &'static str,
    /// Role grant via membership; `None` exercises permission grants
    /// without any role in play.
    role: Option<&'static Role>,
    /// Direct `user_permission_grants` on top of (or instead of) the role.
    permissions: &'static [&'static Permission],
}

const USERS: [User; 3] = [
    // role grants only
    User {
        slot: 10,
        username: "seedz:viewer",
        email: "viewer@seedz.local",
        first_name: "Vera",
        last_name: "Viewer",
        role: Some(&ROLE_VIEWER),
        permissions: &[],
    },
    // role + a direct grant beyond the role
    User {
        slot: 11,
        username: "seedz:editor",
        email: "editor@seedz.local",
        first_name: "Eli",
        last_name: "Editor",
        role: Some(&ROLE_EDITOR),
        permissions: &[&DEMO_REPORTS_EXPORT],
    },
    // direct grants only, no role
    User {
        slot: 12,
        username: "seedz:auditor",
        email: "auditor@seedz.local",
        first_name: "Amara",
        last_name: "Auditor",
        role: None,
        permissions: &[&DEMO_AUDIT_READ],
    },
];

const SETTING_KEY: &str = "seedz:sample";
const SETTING_VALUE: &str = r#"{"seeded_by":"seedz","note":"local-dev sample setting"}"#;

/// The demo fixture seeder. Safe to run repeatedly, and safe to run after
/// entities with the same names were created by hand or through the API
/// (their live row ids are resolved and reused; see module docs).
pub struct FixturesSeeder;

#[async_trait]
impl Seeder for FixturesSeeder {
    async fn seed(&self, write: &PgPool, _read: &PgPool) -> Result<(), BoxedError> {
        let authority_id = seed_authority(write).await?;

        // Natural keys first, ids resolved once: every grant below points
        // at LIVE row ids, whether the row is ours or pre-existing.
        let mut permission_ids = HashMap::new();

        for permission in [
            &DEMO_DASHBOARD_VIEW,
            &DEMO_DASHBOARD_EDIT,
            &DEMO_REPORTS_EXPORT,
            &DEMO_AUDIT_READ,
        ] {
            permission_ids.insert(permission.key(), seed_permission(write, permission).await?);
        }

        let mut role_ids = HashMap::new();

        for role in ROLES {
            role_ids.insert(role.name, seed_role(write, role, &permission_ids).await?);
        }

        for user in USERS {
            seed_user(write, &user, authority_id, &role_ids, &permission_ids).await?;
        }

        seed_setting(write).await?;

        Ok(())
    }
}

/// Seeds the `local-dev` realm; returns its live row id.
async fn seed_authority(write: &PgPool) -> Result<Uuid, BoxedError> {
    const NAME: &str = "local-dev";

    if let Some(id) = find_id(
        write,
        "SELECT id FROM authorities WHERE name = $1",
        &[Name(NAME)],
    )
    .await?
    {
        info!("authority {NAME} already exists -- reusing id {id}");

        return Ok(id);
    }

    let id = seed_uuid(AUTHORITY_LOCAL_DEV_ID);

    sqlx::query(
        "INSERT INTO authorities (id, client_key, name, status, strategy, settings, params)
         VALUES ($1, $2, $3, 'enabled', 'username_password', $4::jsonb, $5::jsonb)",
    )
    .bind(id)
    .bind(seed_uuid(AUTHORITY_LOCAL_DEV_CLIENT_KEY))
    .bind(NAME)
    .bind(AUTHORITY_SETTINGS)
    .bind(AUTHORITY_PARAMS)
    .execute(write)
    .await?;

    info!("authority {NAME} created");

    Ok(id)
}

/// Seeds one permission; returns its live row id.
async fn seed_permission(write: &PgPool, permission: &Permission) -> Result<Uuid, BoxedError> {
    if let Some(id) = find_id(
        write,
        "SELECT id FROM permissions WHERE realm = $1 AND resource = $2 AND action = $3",
        &[
            Name(permission.realm),
            Name(permission.resource),
            Name(permission.action),
        ],
    )
    .await?
    {
        info!(
            "permission {} already exists -- reusing id {}",
            permission.key(),
            id
        );

        return Ok(id);
    }

    let id = seed_uuid(permission.slot);

    sqlx::query("INSERT INTO permissions (id, realm, resource, action) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(permission.realm)
        .bind(permission.resource)
        .bind(permission.action)
        .execute(write)
        .await?;

    info!("permission {} created", permission.key());

    Ok(id)
}

/// Seeds one role and its role-permission grants; returns its live row id.
async fn seed_role(
    write: &PgPool,
    role: &Role,
    permission_ids: &HashMap<String, Uuid>,
) -> Result<Uuid, BoxedError> {
    let role_id = if let Some(id) = find_id(
        write,
        "SELECT id FROM roles WHERE name = $1",
        &[Name(role.name)],
    )
    .await?
    {
        info!("role {} already exists -- reusing id {}", role.name, id);

        id
    } else {
        let id = seed_uuid(role.slot);

        sqlx::query("INSERT INTO roles (id, name) VALUES ($1, $2)")
            .bind(id)
            .bind(role.name)
            .execute(write)
            .await?;

        info!("role {} created", role.name);

        id
    };

    for permission in role.permissions {
        grant(
            write,
            "role_permission_grants",
            "role_id",
            "permission_id",
            role_id,
            resolved_permission(permission, permission_ids)?,
        )
        .await?;
    }

    Ok(role_id)
}

/// Seeds one user, its realm attachment, and its role/permission grants.
async fn seed_user(
    write: &PgPool,
    user: &User,
    authority_id: Uuid,
    role_ids: &HashMap<&str, Uuid>,
    permission_ids: &HashMap<String, Uuid>,
) -> Result<(), BoxedError> {
    let user_id = if let Some(id) = find_id(
        write,
        "SELECT id FROM users WHERE username = $1",
        &[Name(user.username)],
    )
    .await?
    {
        info!("user {} already exists -- reusing id {}", user.username, id);

        id
    } else {
        let id = seed_uuid(user.slot);

        sqlx::query(
            "INSERT INTO users (id, kind, status, username, email, first_name, last_name)
             VALUES ($1, 'human', 'enabled', $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(user.username)
        .bind(user.email)
        .bind(user.first_name)
        .bind(user.last_name)
        .execute(write)
        .await?;

        info!("user {} created", user.username);

        id
    };

    // Attach to the demo realm (bootstrap's default realm is untouched);
    // user_identifier mirrors what registration stores: the username.
    if !row_exists(
        write,
        "SELECT EXISTS(SELECT 1 FROM user_authorities WHERE user_id = $1 AND authority_id = $2)",
        &[Id(user_id), Id(authority_id)],
    )
    .await?
    {
        sqlx::query(
            "INSERT INTO user_authorities (user_id, authority_id, user_identifier)
             VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind(authority_id)
        .bind(user.username)
        .execute(write)
        .await?;

        info!("user {} attached to authority local-dev", user.username);
    }

    if let Some(role) = user.role {
        let role_id = role_ids
            .get(role.name)
            .copied()
            .ok_or_else(|| format!("role {} was not seeded", role.name))?;

        grant(
            write,
            "user_role_grants",
            "user_id",
            "role_id",
            user_id,
            role_id,
        )
        .await?;
    }

    for permission in user.permissions {
        grant(
            write,
            "user_permission_grants",
            "user_id",
            "permission_id",
            user_id,
            resolved_permission(permission, permission_ids)?,
        )
        .await?;
    }

    Ok(())
}

async fn seed_setting(write: &PgPool) -> Result<(), BoxedError> {
    if row_exists(
        write,
        "SELECT EXISTS(SELECT 1 FROM settings WHERE key = $1)",
        &[Name(SETTING_KEY)],
    )
    .await?
    {
        info!("setting {SETTING_KEY} already seeded -- leaving its value alone");

        return Ok(());
    }

    sqlx::query("INSERT INTO settings (key, value) VALUES ($1, $2::jsonb)")
        .bind(SETTING_KEY)
        .bind(SETTING_VALUE)
        .execute(write)
        .await?;

    info!("setting {SETTING_KEY} created");

    Ok(())
}

/// Lookup key into the resolved permission-id map.
fn resolved_permission(
    permission: &Permission,
    permission_ids: &HashMap<String, Uuid>,
) -> Result<Uuid, BoxedError> {
    permission_ids
        .get(&permission.key())
        .copied()
        .ok_or_else(|| format!("permission {} was not seeded", permission.key()).into())
}

/// Bind payload for the small find probes: text or uuid.
#[derive(Clone, Copy)]
enum Param<'a> {
    Name(&'a str),
    Id(Uuid),
}

use Param::{Id, Name};

/// `SELECT id ...` probe: the live row's id when the natural key exists.
async fn find_id(
    write: &PgPool,
    sql: &str,
    params: &[Param<'_>],
) -> Result<Option<Uuid>, BoxedError> {
    let mut query = sqlx::query_scalar::<_, Uuid>(sql);

    for param in params {
        query = match param {
            Name(value) => query.bind(value),
            Id(value) => query.bind(value),
        };
    }

    Ok(query
        .fetch_optional(write)
        .await?)
}

async fn row_exists(write: &PgPool, sql: &str, params: &[Param<'_>]) -> Result<bool, BoxedError> {
    let mut query = sqlx::query_scalar::<_, bool>(sql);

    for param in params {
        query = match param {
            Name(value) => query.bind(value),
            Id(value) => query.bind(value),
        };
    }

    Ok(query.fetch_one(write).await?)
}

/// Find-then-insert for a composite-PK grant row.
async fn grant(
    write: &PgPool,
    table: &str,
    left: &str,
    right: &str,
    left_id: Uuid,
    right_id: Uuid,
) -> Result<(), BoxedError> {
    let sql = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE {left} = $1 AND {right} = $2)");

    let exists = row_exists(write, &sql, &[Id(left_id), Id(right_id)]).await?;

    if exists {
        return Ok(());
    }

    let sql = format!("INSERT INTO {table} ({left}, {right}) VALUES ($1, $2)");

    sqlx::query(&sql)
        .bind(left_id)
        .bind(right_id)
        .execute(write)
        .await?;

    info!("granted {right} on {table}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    // NOTE on the determinism split: there is no seed parameter and no
    // randomness anywhere in this module — every id derives from a const
    // `slot` via `seed_uuid`, so "same-seed runs equal" holds for ALL runs
    // by construction and "seedless runs differ" has no API to test. The
    // async helpers (seed_authority, seed_permission, seed_role, seed_user,
    // seed_setting, find_id, row_exists, grant, FixturesSeeder::seed) all
    // take &PgPool and are exercised only by real `seedz` runs against a
    // local DB; the pure preconditions they rely on (map keys covering
    // every lookup, collision-free fresh-insert PKs) are pinned here.

    const ALL_PERMISSIONS: [&'static Permission; 4] = [
        &DEMO_DASHBOARD_VIEW,
        &DEMO_DASHBOARD_EDIT,
        &DEMO_REPORTS_EXPORT,
        &DEMO_AUDIT_READ,
    ];

    #[test]
    fn seed_uuid_is_deterministic_and_format_pinned() {
        // Repeated calls for the same slot are equal: pure function of slot.
        for slot in [0u8, 1, 11, 42, 255] {
            assert_eq!(seed_uuid(slot), seed_uuid(slot));
        }

        // Exact format pinned: the `f0…{slot:02x}` layout is a contract
        // pinned so probes can target the client key uuid verbatim.
        assert_eq!(
            seed_uuid(AUTHORITY_LOCAL_DEV_ID)
                .as_hyphenated()
                .to_string(),
            "f0000000-0000-4000-8000-000000000001"
        );
        assert_eq!(
            seed_uuid(AUTHORITY_LOCAL_DEV_CLIENT_KEY)
                .as_hyphenated()
                .to_string(),
            "f0000000-0000-4000-8000-000000000002"
        );
        assert_eq!(
            seed_uuid(255)
                .as_hyphenated()
                .to_string(),
            "f0000000-0000-4000-8000-0000000000ff"
        );
        assert_eq!(
            seed_uuid(0)
                .as_hyphenated()
                .to_string(),
            "f0000000-0000-4000-8000-000000000000"
        );

        assert_eq!(seed_uuid(10).get_version(), Some(uuid::Version::Random));
        assert_eq!(seed_uuid(10).get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn seed_uuid_is_injective_over_the_whole_slot_space() {
        let ids: HashSet<Uuid> = (0..=u8::MAX)
            .map(seed_uuid)
            .collect();
        assert_eq!(ids.len(), usize::from(u8::MAX) + 1);
    }

    #[test]
    fn seed_slots_stay_in_their_documented_disjoint_bands() {
        assert!((1..=2).contains(&AUTHORITY_LOCAL_DEV_ID));
        assert!((1..=2).contains(&AUTHORITY_LOCAL_DEV_CLIENT_KEY));
        assert!(
            USERS
                .iter()
                .all(|u| (10..=19).contains(&u.slot))
        );
        assert!(
            ALL_PERMISSIONS
                .iter()
                .all(|p| (30..=39).contains(&p.slot))
        );
        assert!(
            ROLES
                .iter()
                .all(|r| (40..=49).contains(&r.slot))
        );
    }

    #[test]
    fn permission_key_is_realm_resource_action() {
        assert_eq!(DEMO_DASHBOARD_VIEW.key(), "demo:dashboard:view");
        assert_eq!(DEMO_REPORTS_EXPORT.key(), "demo:reports:export");

        // The key doubles as the resolved-permission map key, so the seed
        // rows need distinct natural keys or one row would overwrite another.
        let keys: HashSet<String> = ALL_PERMISSIONS
            .iter()
            .map(|p| p.key())
            .collect();
        assert_eq!(keys.len(), ALL_PERMISSIONS.len());
    }

    #[test]
    fn resolved_permission_hits_by_key_and_errors_by_key() {
        let mut ids = HashMap::new();
        ids.insert(
            DEMO_DASHBOARD_VIEW.key(),
            seed_uuid(DEMO_DASHBOARD_VIEW.slot),
        );

        assert_eq!(
            resolved_permission(&DEMO_DASHBOARD_VIEW, &ids).unwrap(),
            seed_uuid(30)
        );

        let err = resolved_permission(&DEMO_AUDIT_READ, &ids).unwrap_err();
        assert!(
            err.to_string()
                .contains("demo:audit:read was not seeded"),
            "unhelpful error: {err}"
        );
    }

    /// Mirrors the fresh-insert path of `FixturesSeeder::seed`: the maps it
    /// builds (keyed by natural key) must cover every lookup `seed_role` /
    /// `seed_user` later performs, grant targets must equal the parent ids
    /// seeded in the same run, and no two graph nodes may share a PK.
    fn build_run_maps() -> (HashMap<String, Uuid>, HashMap<&'static str, Uuid>) {
        let mut permission_ids = HashMap::new();

        for permission in ALL_PERMISSIONS {
            assert!(
                permission_ids
                    .insert(permission.key(), seed_uuid(permission.slot))
                    .is_none(),
                "duplicate permission natural key would silently drop a seed row"
            );
        }

        let mut role_ids = HashMap::new();

        for role in ROLES {
            assert!(
                role_ids
                    .insert(role.name, seed_uuid(role.slot))
                    .is_none(),
                "duplicate role name would silently drop a seed row"
            );
        }

        (permission_ids, role_ids)
    }

    #[test]
    fn fixture_graph_is_fk_consistent() {
        let (permission_ids, role_ids) = build_run_maps();

        // seed_role precondition: every role permission resolves in the map.
        for role in ROLES {
            for permission in role.permissions {
                assert!(
                    permission_ids.contains_key(&permission.key()),
                    "role {} references unseeded permission {}",
                    role.name,
                    permission.key()
                );
            }
        }

        // seed_user preconditions + child id == parent id from this run.
        for user in USERS {
            if let Some(role) = user.role {
                let id = *role_ids
                    .get(role.name)
                    .unwrap_or_else(|| {
                        panic!(
                            "user {} references unseeded role {}",
                            user.username, role.name
                        )
                    });
                assert_eq!(id, seed_uuid(role.slot));
            }

            for permission in user.permissions {
                let id = resolved_permission(permission, &permission_ids)
                    .expect("user direct grant must resolve");
                assert_eq!(id, seed_uuid(permission.slot));
            }
        }
    }

    #[test]
    fn fresh_insert_graph_nodes_are_collision_free() {
        let ids: HashSet<Uuid> = [
            seed_uuid(AUTHORITY_LOCAL_DEV_ID),
            seed_uuid(AUTHORITY_LOCAL_DEV_CLIENT_KEY),
        ]
        .into_iter()
        .chain(
            USERS
                .iter()
                .map(|u| seed_uuid(u.slot)),
        )
        .chain(
            ROLES
                .iter()
                .map(|r| seed_uuid(r.slot)),
        )
        .chain(
            ALL_PERMISSIONS
                .iter()
                .map(|p| seed_uuid(p.slot)),
        )
        .collect();

        assert_eq!(
            ids.len(),
            2 + USERS.len() + ROLES.len() + ALL_PERMISSIONS.len()
        );
    }

    #[test]
    fn usernames_and_emails_are_unique() {
        let usernames: HashSet<&str> = USERS
            .iter()
            .map(|u| u.username)
            .collect();
        assert_eq!(usernames.len(), USERS.len());

        let emails: HashSet<&str> = USERS
            .iter()
            .map(|u| u.email)
            .collect();
        assert_eq!(emails.len(), USERS.len());
    }

    #[test]
    fn graph_build_is_deterministic_across_runs() {
        // Two full "builder runs" produce identical maps and identical
        // grant-target ids: re-seeding converges on the same rows.
        let (perms_a, roles_a) = build_run_maps();
        let (perms_b, roles_b) = build_run_maps();

        assert_eq!(perms_a, perms_b);
        assert_eq!(roles_a, roles_b);
    }
}
