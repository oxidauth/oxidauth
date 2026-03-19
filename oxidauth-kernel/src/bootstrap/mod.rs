use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::dev_prelude::BoxedError;

#[async_trait]
pub trait BootstrapTrait: Send + Sync + 'static {
    async fn bootstrap(
        &self,
        params: &BootstrapParams,
    ) -> Result<(), BoxedError>;
}

pub type BootstrapService = Arc<dyn BootstrapTrait>;

#[derive(Debug, Deserialize)]
pub struct BootstrapParams;
