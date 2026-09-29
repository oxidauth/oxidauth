pub mod middleware;
pub mod provider;
pub mod response;
pub mod server;

use oxidauth_kernel::bootstrap::{BootstrapParams, BootstrapService};
use oxidauth_usecases::bootstrap::SudoUserBootstrapUseCase;
use std::{error::Error, sync::Arc};
use tracing::info;

use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    println!("engaging oxidauth http server...");

    let subscriber = oxidauth_telemetry::get_subscriber(
        "oxidauth-http-api".into(),
        "INFO".into(),
        std::io::stdout,
    );

    oxidauth_telemetry::init_subscriber(subscriber);

    let provider = provider::init().await?;

    let bootstrap: BootstrapService = Arc::new(SudoUserBootstrapUseCase::new(
        &provider,
    ));

    bootstrap
        .call(&BootstrapParams)
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
