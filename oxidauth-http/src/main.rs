pub mod provider;
pub mod response;
pub mod server;

use std::error::Error;

use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let addr = "0.0.0.0:80".parse()?;

    let provider = provider::setup();

    let server = Server::new(addr, provider);

    server.start().await?;

    Ok(())
}
