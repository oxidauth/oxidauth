use oxidauth_http::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:9120";

    let server = Server::new(addr)?;

    server.start().await?;

    Ok(())
}
