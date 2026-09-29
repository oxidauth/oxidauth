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
