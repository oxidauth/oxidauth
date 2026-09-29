use crate::dev_prelude::*;

pub use super::PrivateKey;

pub type FindMostRecentPrivateKeyService = Arc<
    dyn for<'a> Service<
        &'a FindMostRecentPrivateKey,
        Response = PrivateKey,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
pub struct FindMostRecentPrivateKey {}
