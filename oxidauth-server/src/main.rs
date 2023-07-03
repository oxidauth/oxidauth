#![allow(unused)]

use std::env::var;
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

    let bind_addr = var("BIND_ADDR")?.parse()?;
    let database_url = var("DATABASE_URL")?;

    let database = PgPoolOptions::new().connect(&database_url).await?;

    axum::Server::new()
        .address(bind_addr)
        .database(database)
        .build()?
        .start()
        .await?;

    Ok(())
}
