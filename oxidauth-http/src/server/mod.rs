pub mod api;

use std::{error::Error, net::SocketAddr};

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::provider::Provider;

pub struct Server {
    addr: SocketAddr,
    provider: Provider,
}

impl Server {
    pub fn new(addr: SocketAddr, provider: Provider) -> Self {
        Self { addr, provider }
    }

    pub async fn start(
        self,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let tcp_listener = TcpListener::bind(self.addr).await?;

        axum::serve(
            tcp_listener,
            router(self.provider).into_make_service(),
        )
        .await?;

        Ok(())
    }
}

pub fn router(provider: Provider) -> Router {
    Router::new()
        .nest("/api", api::router())
        // TODO(drewbrad4): replace with something more restrictive
        // https://www.pivotaltracker.com/story/show/186909011
        .layer(CorsLayer::permissive())
        .with_state(provider)
}
