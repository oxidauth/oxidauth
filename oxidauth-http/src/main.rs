pub mod provider;
pub mod response;
pub mod server;

use std::error::Error;
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

    info!("starting server...");

    let addr = "0.0.0.0:80".parse()?;

    let provider = provider::setup().await?;

    let server = Server::new(addr, provider);

    info!("http booting");

    server.start().await?;

    Ok(())
}
