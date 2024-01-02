pub mod api;

use std::{error::Error, net::SocketAddr};

use axum::Router;

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
        axum::Server::bind(&self.addr)
            .serve(router(self.provider).into_make_service())
            .await?;

        Ok(())
    }
}

pub fn router(provider: Provider) -> Router {
    Router::new()
        .nest("/api", api::router())
        .with_state(provider)
}
