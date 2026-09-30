pub mod middleware;
pub mod provider;
pub mod server;

use std::{error::Error, sync::Arc};

use oxidauth_kernel::bootstrap::{BootstrapParams, BootstrapService};
use oxidauth_services::bootstrap::SudoUserBootstrapUseCase;
use server::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let (environment, tracing_level) = telemetry::get_logging_envs()?;

    let subscriber = telemetry::get_subscriber(
        "oxidauth-api",
        env!("CARGO_PKG_VERSION"),
        &environment,
        &tracing_level,
        std::io::stdout,
    );

    telemetry::init_subscriber(subscriber);

    info!("starting oxidauth-api");

    let provider = provider::init().await?;

    let bootstrap: BootstrapService = Arc::new(SudoUserBootstrapUseCase::new(&provider));

    bootstrap
        .bootstrap(&BootstrapParams)
        .await?;

    info!("starting server...");
    // container default is 80 (compose maps it); override (e.g. `PORT=3002`) when
    // running on the host, where the developer machine's ports are taken.
    let port = std::env::var("PORT").unwrap_or_else(|_| "80".to_string());
    let addr = format!("0.0.0.0:{port}").parse()?;

    let server = Server::new(addr, provider);

    info!("http booting...");
    server.start().await?;

    Ok(())
}
