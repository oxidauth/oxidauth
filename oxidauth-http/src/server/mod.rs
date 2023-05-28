pub mod routes;
pub mod shutdown_signal;

use std::{
    fmt,
    net::{AddrParseError, SocketAddr},
};

use self::shutdown_signal::shutdown_signal;

pub struct Server {
    address: SocketAddr,
}

impl Server {
    pub fn new(address: &str) -> Result<Self, ServerError> {
        let address = match address.parse() {
            Ok(address) => address,
            Err(err) => return Err(ServerError::BadAddress(address.to_owned(), err)),
        };

        Ok(Self { address })
    }

    pub async fn start(&self) -> Result<(), ServerError> {
        axum::Server::bind(&self.address)
            .serve(routes::router().into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(ServerError::ServerStartup)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum ServerError {
    BadAddress(String, AddrParseError),
    ServerStartup(hyper::Error),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadAddress(addr, err) => write!(
                f,
                "server error: unable to parse address: {addr} error: {err}"
            ),
            Self::ServerStartup(err) => write!(f, "server error: unable to start server: {err}"),
        }
    }
}

impl std::error::Error for ServerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_new() {
        assert!(Server::new("0.0").is_err());
        assert!(Server::new("0.0.0.0:0").is_ok());
    }

    // #[tokio::test]
    // async fn server_start() {
    //     let handle = tokio::spawn(async {
    //         let server = Server::new("0.0.0.0:5959").unwrap();
    //         server.start().await.unwrap();
    //     });

    //     let server2 = Server::new("0.0.0.0:5959").unwrap();
    //     assert!(server2.start().await.is_err());
    // }
}
