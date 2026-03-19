use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::BoxedError;

pub use super::Authority;

#[async_trait]
pub trait ListAllAuthoritiesTrait: Send + Sync + 'static {
    async fn list_all_authorities(
        &self,
        params: &ListAllAuthorities,
    ) -> Result<Vec<Authority>, BoxedError>;
}

pub type ListAllAuthoritiesService = Arc<dyn ListAllAuthoritiesTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllAuthorities {}
