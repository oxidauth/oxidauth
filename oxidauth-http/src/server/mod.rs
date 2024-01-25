pub mod api;

use std::{error::Error, net::SocketAddr};

use axum::Router;
use tokio::net::TcpListener;

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
        .with_state(provider)
}
