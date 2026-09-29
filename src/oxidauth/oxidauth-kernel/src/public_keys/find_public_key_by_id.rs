use crate::dev_prelude::*;

pub use super::PublicKey;

pub type FindPublicKeyByIdService = Arc<
    dyn for<'a> Service<
        &'a FindPublicKeyById,
        Response = PublicKey,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Deserialize)]
pub struct FindPublicKeyById {
    pub public_key_id: Uuid,
}
