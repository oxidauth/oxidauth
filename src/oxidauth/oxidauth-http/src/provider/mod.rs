pub mod postgres;
pub mod services;

use oxidauth_kernel::error::BoxedError;
pub use provider::Provider;

pub async fn init() -> Result<Provider, BoxedError> {
    let mut provider = Provider::new();

    postgres::init(&mut provider).await?;
    services::init(&mut provider).await?;

    Ok(provider)
}
