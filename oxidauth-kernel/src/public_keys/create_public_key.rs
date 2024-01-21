use crate::dev_prelude::*;

pub use super::PublicKey;

pub type CreatePublicKeyService = Arc<
    dyn for<'a> Service<
        &'a CreatePublicKey,
        Response = PublicKey,
        Error = BoxedError,
    >,
>;

#[derive(Debug)]
pub struct CreatePublicKey();
