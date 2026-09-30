use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use super::Authority;
use crate::error::BoxedError;

#[async_trait]
pub trait ListAllAuthoritiesServiceTrait: Send + Sync + 'static {
    async fn list_all_authorities(
        &self,
        params: &ListAllAuthorities,
    ) -> Result<Vec<Authority>, BoxedError>;
}

pub type ListAllAuthoritiesService = Arc<dyn ListAllAuthoritiesServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllAuthorities {}
