use std::sync::Arc;

use serde::Deserialize;

use crate::error::BoxedError;
pub use crate::service::Service;

pub use super::Authority;

pub type ListAllAuthoritiesService = Arc<
    dyn for<'a> Service<
        &'a ListAllAuthorities,
        Response = Vec<Authority>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct ListAllAuthorities {}
