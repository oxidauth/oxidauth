use std::{env, process};

use oxidauth_kernel::error::BoxedError;
use oxidauth_postgres::Database;
use seedz::{SeedRunner, fixtures::FixturesSeeder};

const ENVIRONMENT: &str = "ENVIRONMENT";

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    // Dev-only guard FIRST (before telemetry, before any DB handle): seedz
    // writes fixture rows into a real auth database and must never aim at
    // staging/production by accident. First-run provisioning is bootstrap's
    // job (oxidauth-api boot), not this binary's.
    let environment = env::var(ENVIRONMENT).unwrap_or_else(|_| "<unset>".to_string());

    if environment != "local" {
        eprintln!(
            "seedz is for local development; refusing to run against ENVIRONMENT={environment}"
        );

        process::exit(1);
    }

    let (environment, tracing_level) = telemetry::get_logging_envs()?;

    let subscriber = telemetry::get_subscriber(
        "seedz",
        env!("CARGO_PKG_VERSION"),
        &environment,
        &tracing_level,
        std::io::stdout,
    );

    telemetry::init_subscriber(subscriber);

    // DATABASE_URL (required) + READ_DATABASE_URL (optional) from env,
    // plan-05 Database; migrate() honors MIGRATIONS_ENABLED.
    let database = Database::from_env().await?;

    database.migrate().await?;

    let mut runner = SeedRunner::new();

    runner.add_seeder(Box::new(FixturesSeeder));

    runner
        .run(&database.write_pool(), &database.read_pool())
        .await
}
