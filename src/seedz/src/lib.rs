//! Project-level seeder (template `src/seedz` API: `Seeder` trait +
//! `SeedRunner`).
//!
//! **Local development fixtures only.** The `seedz` bin refuses to run
//! unless `ENVIRONMENT=local`. First-run provisioning — JWT keys, the
//! `oxidauth:admin` role/user, the default `username_password` authority,
//! the `bootstrap` setting — belongs to `oxidauth_services::bootstrap`,
//! invoked from `oxidauth-api` boot; seeding those here would fight the
//! provisioning path, so this crate deliberately never touches them.

pub mod fixtures;

use async_trait::async_trait;
pub use oxidauth_kernel::error::BoxedError;
use sqlx::PgPool;
use tracing::info;

#[async_trait]
pub trait Seeder: Send + Sync {
    async fn seed(&self, write: &PgPool, read: &PgPool) -> Result<(), BoxedError>;
}

pub struct SeedRunner {
    seeders: Vec<Box<dyn Seeder + Send + Sync>>,
}

impl Default for SeedRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl SeedRunner {
    pub fn new() -> Self {
        Self {
            seeders: Vec::new(),
        }
    }

    pub fn add_seeder(&mut self, seeder: Box<dyn Seeder + Send + Sync>) {
        self.seeders.push(seeder);
    }

    pub async fn run(&self, write: &PgPool, read: &PgPool) -> Result<(), BoxedError> {
        info!("Running seeders...");

        for seeder in &self.seeders {
            seeder
                .seed(write, read)
                .await?;
        }

        info!("All seeders completed successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use super::*;

    // `connect_lazy_with` builds a pool without ever dialing the DB, and
    // these seeders never query — the whole orchestration is observable
    // offline. `FixturesSeeder::seed` (the DB-touching seeder) is exercised
    // only by real `seedz` runs; here we cover `Seeder` (via a recorder),
    // `SeedRunner::{new, default, add_seeder, run}`.
    fn offline_pool() -> PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy_with(sqlx::postgres::PgConnectOptions::new())
    }

    struct Recorder {
        name: &'static str,
        fail: bool,
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl Recorder {
        fn ok(name: &'static str, calls: &Arc<Mutex<Vec<&'static str>>>) -> Box<Self> {
            Box::new(Self {
                name,
                fail: false,
                calls: Arc::clone(calls),
            })
        }

        fn failing(name: &'static str, calls: &Arc<Mutex<Vec<&'static str>>>) -> Box<Self> {
            Box::new(Self {
                name,
                fail: true,
                calls: Arc::clone(calls),
            })
        }
    }

    #[async_trait]
    impl Seeder for Recorder {
        async fn seed(&self, _write: &PgPool, _read: &PgPool) -> Result<(), BoxedError> {
            self.calls
                .lock()
                .await
                .push(self.name);

            if self.fail {
                return Err(format!("{name} failed", name = self.name).into());
            }

            Ok(())
        }
    }

    #[tokio::test]
    async fn empty_runner_completes_without_touching_the_database() {
        let pool = offline_pool();

        SeedRunner::new()
            .run(&pool, &pool)
            .await
            .unwrap();
        SeedRunner::default()
            .run(&pool, &pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn add_seeder_runs_seeders_in_insertion_order() {
        let pool = offline_pool();
        let calls = Arc::new(Mutex::new(Vec::new()));

        let mut runner = SeedRunner::new();
        runner.add_seeder(Recorder::ok("first", &calls));
        runner.add_seeder(Recorder::ok("second", &calls));

        runner
            .run(&pool, &pool)
            .await
            .unwrap();

        assert_eq!(*calls.lock().await, vec!["first", "second"]);
    }

    #[tokio::test]
    async fn run_propagates_the_first_seeder_error_and_skips_the_rest() {
        let pool = offline_pool();
        let calls = Arc::new(Mutex::new(Vec::new()));

        let mut runner = SeedRunner::new();
        runner.add_seeder(Recorder::ok("first", &calls));
        runner.add_seeder(Recorder::failing("boom", &calls));
        runner.add_seeder(Recorder::ok("never", &calls));

        let err = runner
            .run(&pool, &pool)
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), "boom failed");
        assert_eq!(*calls.lock().await, vec!["first", "boom"]);
    }
}
