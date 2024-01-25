use std::sync::Arc;

use serde::Deserialize;

use crate::dev_prelude::{BoxedError, Service};

pub type BootstrapService = Arc<
    dyn for<'a> Service<&'a BootstrapParams, Response = (), Error = BoxedError>,
>;

#[derive(Debug, Deserialize)]
pub struct BootstrapParams;
