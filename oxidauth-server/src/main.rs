#![allow(unused)]

use std::error::Error;

use sqlx::postgres::PgPoolOptions;

pub mod authorities;
pub mod axum;
pub mod error;
pub mod jwt;
pub mod prelude;
pub mod rsa;

pub type BoxedError = Box<(dyn Error + Send + Sync + 'static)>;

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    dotenv::dotenv()?;

    let address = "0.0.0.0:3000".parse()?;

    let database = PgPoolOptions::new()
        .connect("postgres://oxidauth:oxidauth@127.0.0.1:5432/oxidauth")
        .await?;

    axum::Server::new()
        .address(address)
        .database(database)
        .build()?
        .start()
        .await?;

    Ok(())
}
