use std::{fmt::Display, net::SocketAddr};

use axum::Server as AxumServer;
use sqlx::PgPool;

use crate::BoxedError;

use super::routes;

pub struct Server {
    address: SocketAddr,
    database: PgPool,
}

impl Server {
    pub fn new() -> ServerBuilder {
        ServerBuilder::default()
    }

    pub async fn start(&self) -> Result<(), BoxedError> {
        let _result = AxumServer::bind(&self.address)
            .serve(routes::router(&self.database).into_make_service())
            .await;

        Ok(())
    }
}

#[derive(Default)]
pub struct ServerBuilder {
    address: Option<SocketAddr>,
    database: Option<PgPool>,
}

impl ServerBuilder {
    pub fn build(self) -> Result<Server, ServerBuildError> {
        self.check()?;

        let address = self.address.unwrap();
        let database = self.database.unwrap();

        let server = Server { address, database };

        Ok(server)
    }

    fn check(&self) -> Result<(), ServerBuildError> {
        let mut errors = ServerBuildError::new();

        if self.address.is_none() {
            errors.push(Box::new(ServerMissingFieldError::Address));
        }

        if self.database.is_none() {
            errors.push(Box::new(ServerMissingFieldError::Database));
        }

        if errors.is_some() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    pub fn address(mut self, address: SocketAddr) -> Self {
        self.address = Some(address);

        self
    }

    pub fn database(mut self, database: PgPool) -> Self {
        self.database = Some(database);

        self
    }
}

#[derive(Debug)]
pub struct ServerBuildError {
    errors: Vec<BoxedError>,
}

impl ServerBuildError {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn push(&mut self, err: BoxedError) {
        self.errors.push(err);
    }

    pub fn is_some(&self) -> bool {
        self.errors.len() > 0
    }
}

impl Display for ServerBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errors: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();

        write!(f, "unable to build server: {}", errors.join(", "))
    }
}

impl std::error::Error for ServerBuildError {}

#[derive(Debug)]
pub enum ServerMissingFieldError {
    Address,
    Database,
}

impl Display for ServerMissingFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Address => write!(f, "missing address"),
            Self::Database => write!(f, "missing database"),
        }
    }
}

impl std::error::Error for ServerMissingFieldError {}
