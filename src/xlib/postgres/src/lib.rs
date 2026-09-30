//! Vendored from freshbrewlabs project-template @ c38ec0a (2026-09); re-diff when templates change.
use std::{env::VarError, error::Error, fmt, sync::Arc};

use async_trait::async_trait;

pub mod ping;

#[async_trait]
pub trait PingTrait: Send + Sync + 'static {
    async fn ping(&self) -> Result<(), PgError>;
}

pub type Ping = Arc<dyn PingTrait>;

/// database macro for instantiating a Database struct and supporting trait impls and methods
///
/// example basic usage:
/// basic usage instantiates a pub const MIGRATOR that can be imported by other crates for tests
/// ```rust,ignore
/// const DATABASE_URL: &str = "DATABASE_URL";
/// const READ_DATABASE_URL: &str = "READ_DATABASE_URL";
/// const MIGRATIONS_ENABLED: &str = "MIGRATIONS_ENABLED";
///
/// database!(DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED);
/// ```
///
/// advanced usage:
/// passing in a sqlx::migrate::Migrator const will skip the pub const MIGRATOR declaration inside
/// the macro. This allows a finer control, but isn't always necessary.
/// ```rust,ignore
/// const DATABASE_URL: &str = "DATABASE_URL";
/// const READ_DATABASE_URL: &str = "READ_DATABASE_URL";
/// const MIGRATIONS_ENABLED: &str = "MIGRATIONS_ENABLED";
/// const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./path/to/migrations");
///
/// database!(DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED, MIGRATOR);
/// ```
///
/// environment vars can be customized
/// in this example FOO env for write db env var
/// in this example BAR env for read db env var
/// in this example BAZ env for migrations enabled env var
/// ```rust,ignore
/// const FOO: &str = "FOO";
/// const BAR: &str = "BAR";
/// const BAZ: &str = "BAZ";
///
/// database!(FOO, BAR, BAZ);
/// ```
///
/// A crate can use more than one Database by using modules
/// ```rust,ignore
/// mod velocity {
///     const DATABASE_URL: &str = "VELOCITY_DATABASE_URL";
///     const READ_DATABASE_URL: &str = "VELOCITY_READ_DATABASE_URL";
///     const MIGRATIONS_ENABLED: &str = "VELOCITY_MIGRATIONS_ENABLED";
///     pub const VELOCITY_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../path/to/migrations");
///
///     database!(DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED, VELOCITY_MIGRATOR);
/// }
///
/// mod events {
///     const DATABASE_URL: &str = "EVENTS_DATABASE_URL";
///     const READ_DATABASE_URL: &str = "EVENTS_READ_DATABASE_URL";
///     const MIGRATIONS_ENABLED: &str = "EVENTS_MIGRATIONS_ENABLED";
///     pub const EVENTS_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../path/to/migrations");
///
///     database!(DATABASE_URL, READ_DATABASE_URL, MIGRATIONS_ENABLED, EVENTS_MIGRATOR);
/// }
///
/// use events::Database as EventsDatabase;
/// use velocity::Database as VelocityDatabase;
///
/// fn main() {
///     let events_db = EventsDatabase::from_env().await?;
///     let velocity_db = VelocityDatabase::from_env().await?;
/// }
///
/// mod tests {
///     use crate::velocity::VELOCITY_MIGRATOR;
///     use sqlx::PgPool;
///
///     #[sqlx::test(VELOCITY_MIGRATOR);
///     async fn test_thing(pool: PgPool) {
///         assert!(true);
///     }
/// }
/// ```
#[macro_export]
macro_rules! database {
    (
        $database_url:expr,
        $read_database_url:expr,
        $migrations_enabled:expr
        $(, $migrator:ident )?
        $(,)?
    ) => {
        use std::env;

        use async_trait::async_trait;
        use $crate::ping;
        pub use $crate::{PgError, Ping, PingTrait, mock};
        use sqlx::{PgPool, migrate::Migrator};

        $crate::migrator_const!($( $migrator )?);

        #[derive(Debug, Clone)]
        pub struct Database {
            write: PgPool,
            read: PgPool,
        }

        impl Database {
            #[tracing::instrument(name = "initialize database from env vars", level = "info")]
            pub async fn from_env() -> Result<Self, PgError> {
                let database_url = env::var($database_url)
                    .map_err(|err| PgError::MissingEnvVar($database_url, err))?;
                let read_database_url = env::var($read_database_url).ok();

                Database::from_database_url(&database_url, read_database_url.as_ref()).await
            }

            pub fn from_pool(pool: sqlx::PgPool) -> Self {
                Self {
                    write: pool.clone(),
                    read: pool.clone(),
                }
            }

            #[tracing::instrument(name = "database from url", level = "debug")]
            pub async fn from_database_url(
                write_url: &String,
                read_url: Option<&String>,
            ) -> Result<Self, PgError> {
                let write = PgPool::connect(&write_url).await?;

                let read_url = read_url.unwrap_or_else(|| write_url);

                let read = PgPool::connect(&read_url).await?;

                Ok(Database { write, read })
            }

            #[tracing::instrument(name = "initialize database from env vars", skip(self), level = "info")]
            pub async fn migrate(&self) -> Result<(), PgError> {
                let migrations_enabled = env::var($migrations_enabled)
                    .map_err(|err| PgError::MissingEnvVar($migrations_enabled, err))?;

                if migrations_enabled != "true" {
                    println!("migrations are not enabled");

                    return Ok(());
                }

                $crate::migrator!($( $migrator )?)
                    .run(&self.write)
                    .await?;

                Ok(())
            }

            pub fn read_pool(&self) -> PgPool {
                self.read.clone()
            }

            pub fn write_pool(&self) -> PgPool {
                self.write.clone()
            }
        }

        #[async_trait]
        impl PingTrait for Database {
            async fn ping(&self) -> Result<(), PgError> {
                ping::ping(&self.write).await?;
                ping::ping(&self.read).await?;

                Ok(())
            }
        }

        #[derive(Default)]
        pub struct DatabaseBuilder {
            database_url: Option<String>,
            read_database_url: Option<String>,
        }

        impl DatabaseBuilder {
            pub fn new() -> Self {
                Default::default()
            }

            pub fn database_url(&mut self, url: String) {
                self.database_url = Some(url);
            }

            pub fn read_database_url(&mut self, url: String) {
                self.read_database_url = Some(url);
            }

            pub async fn build(self) -> Result<Database, PgError> {
                let database_url = self
                    .database_url
                    .ok_or(PgError::MissingDatabaseUrl)?;

                let write = PgPool::connect(&database_url).await?;

                let read_database_url = self
                    .read_database_url
                    .unwrap_or(database_url);

                let read = PgPool::connect(&read_database_url).await?;

                Ok(Database { write, read })
            }
        }
    };
}

#[macro_export]
macro_rules! migrator_const {
    ($migrator:ident) => {};
    () => {
        pub const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();
    };
}

#[macro_export]
macro_rules! migrator {
    ($migrator:ident) => {
        $migrator
    };
    () => {
        sqlx::migrate!()
    };
}

#[derive(Debug)]
pub enum PgError {
    MissingDatabaseUrl,
    MissingEnvVar(&'static str, VarError),
    Sqlx(sqlx::Error),
    FailedMigration(sqlx::migrate::MigrateError),
}

impl fmt::Display for PgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PgError::MissingDatabaseUrl => "missing database url",
            PgError::MissingEnvVar(env_var, err) => {
                &format!("missing {env_var} not found in env: {err:?}")
            },
            PgError::Sqlx(_) => "sqlx err",
            PgError::FailedMigration(_) => "migration failed",
        };

        write!(f, "postgres error: {s}")
    }
}

impl Error for PgError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PgError::MissingDatabaseUrl => None,
            PgError::MissingEnvVar(_, err) => Some(err),
            PgError::Sqlx(err) => Some(err),
            PgError::FailedMigration(err) => Some(err),
        }
    }
}

impl From<sqlx::Error> for PgError {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}

impl From<sqlx::migrate::MigrateError> for PgError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::FailedMigration(err)
    }
}

pub mod mock {
    use super::*;

    pub struct PingMock(pub Result<(), PgError>);

    #[async_trait]
    impl PingTrait for PingMock {
        async fn ping(&self) -> Result<(), PgError> {
            match &self.0 {
                Ok(_) => Ok(()),
                // NOTE(drewbrad4): Had some ownership issues pulling out the
                // exact error passed in. sqlx::Error and the
                // migrate::MigrateError both don't implement Clone.
                Err(_) => Err(PgError::MissingDatabaseUrl),
            }
        }
    }
}

#[cfg(test)]
/// A11.2: structural coverage for the `database!` macro + `PgError` mappings
/// with no live database. Env var names are TEST-only so they can never collide
/// with the real `.env`, and every pool is `connect_lazy` with the maintenance
/// tasks disabled, so no connection is ever attempted.
#[allow(unused_imports)] // `database!` injects its `use` items into this module;
// `Ping`/`migrate::Migrator` are only live for callers
// of the macro outside it
mod db_tests {
    use std::{
        sync::{LazyLock, Mutex, MutexGuard},
        task::{Context, Poll, Waker},
    };

    use super::*;

    // NOTE: deliberately no `use sqlx::PgPool;` here — the `database!` expansion
    // below injects its own `use sqlx::{PgPool, migrate::Migrator};` into this
    // module; a second explicit import would be an E0252 duplicate.

    const DATABASE_URL: &str = "TEST_DATABASE_URL";
    const READ_DATABASE_URL: &str = "TEST_READ_DATABASE_URL";
    const MIGRATIONS_ENABLED: &str = "TEST_MIGRATIONS_ENABLED";

    /// 4-arg macro form: supplying our own migrator skips the macro's
    /// `pub const MIGRATOR` and compiles the EMPTY `./test-migrations` dir, so
    /// nothing here can ever carry a real migration.
    const TEST_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./test-migrations");

    crate::database!(
        DATABASE_URL,
        READ_DATABASE_URL,
        MIGRATIONS_ENABLED,
        TEST_MIGRATOR
    );

    /// Syntactically valid, port-1 endpoint: parses cleanly, never dialed.
    const NEVER_CONNECTS_URL: &str =
        "postgres://oxidauth_test:oxidauth_test@127.0.0.1:1/oxidauth_test_never_connects";

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn env_guard() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Drive a future that resolves without I/O using a no-op waker; panics if
    /// it would block (i.e. try to reach a real database), which these tests
    /// must never do.
    fn poll_ready<T>(fut: impl Future<Output = T>) -> T {
        let mut fut = std::pin::pin!(fut);
        let mut cx = Context::from_waker(Waker::noop());

        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("future would block on a live database; not allowed here"),
        }
    }

    /// Lazy pool with idle/lifetime maintenance disabled: with sqlx's default
    /// timeouts `Pool::new` spawns a reaper task, which needs a tokio runtime
    /// context that plain `#[test]` fns do not provide.
    fn lazy_pool() -> sqlx::PgPool {
        sqlx::pool::PoolOptions::<sqlx::Postgres>::new()
            .max_lifetime(None)
            .idle_timeout(None)
            .connect_lazy(NEVER_CONNECTS_URL)
            .expect("valid but never-dialed pool")
    }

    #[test]
    fn test_migrator_compiles_empty_migration_dir_to_zero_migrations() {
        assert!(
            TEST_MIGRATOR
                .migrations
                .is_empty()
        );
    }

    #[test]
    fn from_pool_wires_read_and_write_pools_without_connecting() {
        let db = Database::from_pool(lazy_pool());

        assert_eq!(
            db.write_pool().size(),
            0,
            "connect_lazy must not open connections"
        );
        assert_eq!(db.read_pool().size(), 0);
        assert_eq!(db.clone().read_pool().size(), 0, "Database must stay Clone");
    }

    #[test]
    fn missing_database_url_display_has_no_source() {
        let err = PgError::MissingDatabaseUrl;

        assert_eq!(err.to_string(), "postgres error: missing database url");
        assert!(err.source().is_none());
    }

    #[test]
    fn missing_env_var_display_and_source_keep_var_name_and_kind() {
        let err = PgError::MissingEnvVar("TEST_DATABASE_URL", VarError::NotPresent);

        assert_eq!(
            err.to_string(),
            "postgres error: missing TEST_DATABASE_URL not found in env: NotPresent"
        );
        assert!(matches!(
            err.source()
                .and_then(|src| src.downcast_ref::<VarError>()),
            Some(VarError::NotPresent)
        ));
    }

    #[test]
    fn sqlx_error_converts_to_sqlx_variant() {
        let err: PgError = sqlx::Error::RowNotFound.into();

        assert!(matches!(err, PgError::Sqlx(sqlx::Error::RowNotFound)));
        assert_eq!(err.to_string(), "postgres error: sqlx err");
        assert!(matches!(
            err.source()
                .and_then(|src| src.downcast_ref::<sqlx::Error>()),
            Some(sqlx::Error::RowNotFound)
        ));
    }

    #[test]
    fn migrate_error_converts_to_failed_migration_variant() {
        // MigrateError is #[non_exhaustive]; its #[from] conversion is the only
        // external constructor, so wrap a sqlx error to get one.
        let migrate_err: sqlx::migrate::MigrateError = sqlx::Error::RowNotFound.into();
        let err: PgError = migrate_err.into();

        assert!(matches!(err, PgError::FailedMigration(_)));
        assert_eq!(err.to_string(), "postgres error: migration failed");
        assert!(
            err.source()
                .and_then(|src| src.downcast_ref::<sqlx::migrate::MigrateError>())
                .is_some()
        );
    }

    #[test]
    fn ping_mock_replays_ok_but_downgrades_any_error() {
        // Pins NOTE(drewbrad4) in `mod mock`: Err payloads always replay as
        // MissingDatabaseUrl because sqlx error types are not Clone.
        assert!(poll_ready(mock::PingMock(Ok(())).ping()).is_ok());

        let failing = mock::PingMock(Err(PgError::Sqlx(sqlx::Error::RowNotFound)));

        match poll_ready(failing.ping()) {
            Err(PgError::MissingDatabaseUrl) => {},
            other => panic!("expected pinned MissingDatabaseUrl downgrade, got {other:?}"),
        }
    }

    #[test]
    fn builder_without_url_fails_before_any_connection_attempt() {
        match poll_ready(DatabaseBuilder::new().build()) {
            Err(err @ PgError::MissingDatabaseUrl) => {
                assert_eq!(err.to_string(), "postgres error: missing database url");
                assert!(err.source().is_none());
            },
            other => panic!("expected MissingDatabaseUrl, got {other:?}"),
        }
    }

    #[test]
    fn builder_setters_store_urls_without_build() {
        // Real coverage for the `database!`-generated `DatabaseBuilder` setters,
        // so the `dead_code` lint keeps biting on genuinely dead helpers instead
        // of a blanket allow. `build()` is never awaited — it would call
        // `PgPool::connect` — the setters are pure string state.
        let mut builder = DatabaseBuilder::new();
        builder.database_url(NEVER_CONNECTS_URL.to_string());
        builder.read_database_url(NEVER_CONNECTS_URL.to_string());

        assert_eq!(
            builder
                .database_url
                .as_deref(),
            Some(NEVER_CONNECTS_URL)
        );
        assert_eq!(
            builder
                .read_database_url
                .as_deref(),
            Some(NEVER_CONNECTS_URL)
        );

        drop(builder);
    }

    #[test]
    fn from_env_without_write_url_reports_missing_env_var() {
        let _guard = env_guard();
        // SAFETY: process-wide env mutation is serialized behind ENV_LOCK.
        unsafe {
            std::env::remove_var(DATABASE_URL);
            std::env::remove_var(READ_DATABASE_URL);
        }

        match poll_ready(Database::from_env()) {
            Err(PgError::MissingEnvVar("TEST_DATABASE_URL", VarError::NotPresent)) => {},
            other => panic!("expected MissingEnvVar(TEST_DATABASE_URL, NotPresent), got {other:?}"),
        }
    }

    #[test]
    fn from_env_requires_write_url_even_when_read_url_is_present() {
        // READ_DATABASE_URL fallback can never rescue a missing write url:
        // the write url is resolved first and short-circuits the fn.
        let _guard = env_guard();
        unsafe {
            std::env::remove_var(DATABASE_URL);
            std::env::set_var(READ_DATABASE_URL, NEVER_CONNECTS_URL);
        }

        let outcome = poll_ready(Database::from_env());

        unsafe { std::env::remove_var(READ_DATABASE_URL) };

        match outcome {
            Err(PgError::MissingEnvVar("TEST_DATABASE_URL", VarError::NotPresent)) => {},
            other => panic!("expected MissingEnvVar(TEST_DATABASE_URL, NotPresent), got {other:?}"),
        }
    }

    #[test]
    fn migrate_without_enabled_flag_reports_missing_env_var() {
        let _guard = env_guard();
        unsafe { std::env::remove_var(MIGRATIONS_ENABLED) };

        let db = Database::from_pool(lazy_pool());

        match poll_ready(db.migrate()) {
            Err(PgError::MissingEnvVar("TEST_MIGRATIONS_ENABLED", VarError::NotPresent)) => {},
            other => {
                panic!("expected MissingEnvVar(TEST_MIGRATIONS_ENABLED, NotPresent), got {other:?}")
            },
        }
    }

    #[test]
    fn migrate_flag_false_is_a_noop_without_touching_the_database() {
        let _guard = env_guard();
        unsafe { std::env::set_var(MIGRATIONS_ENABLED, "false") };

        let db = Database::from_pool(lazy_pool());
        let outcome = poll_ready(db.migrate());

        unsafe { std::env::remove_var(MIGRATIONS_ENABLED) };

        assert!(
            outcome.is_ok(),
            "MIGRATIONS_ENABLED=false must skip running migrations"
        );
    }
}
