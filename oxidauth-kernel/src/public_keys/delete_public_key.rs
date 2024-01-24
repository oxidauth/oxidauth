use crate::dev_prelude::*;

pub use super::PublicKey;

pub type DeletePublicKeyService = Arc<
    dyn for<'a> Service<
        &'a DeletePublicKey,
        Response = PublicKey,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct DeletePublicKey {
    pub public_key_id: Uuid,
}
