use oxidauth_kernel::error::BoxedError;
use oxidauth_postgres::{Database, PingTrait};
use provider::Provider;
use tracing::info;

pub async fn init(provider: &mut Provider) -> Result<(), BoxedError> {
    info!("setting up provider");

    let db = Database::from_env().await?;

    info!("pinging database");

    db.ping().await?;

    info!("attempting to run any migrations");

    db.migrate().await?;

    provider.store::<Database>(db);

    Ok(())
}
