pub mod provider;
pub mod response;
pub mod server;

use std::error::Error;

use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let subscriber = oxidauth_telemetry::get_subscriber(
        "service-http".into(),
        "INFO".into(),
        std::io::stdout,
    );

    oxidauth_telemetry::init_subscriber(subscriber);

    println!("starting server...");

    let addr = "0.0.0.0:80".parse()?;

    let provider = provider::setup().await;

    let server = Server::new(addr, provider);

    println!("http booting");

    server.start().await?;

    Ok(())
}
