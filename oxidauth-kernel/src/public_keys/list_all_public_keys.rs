use crate::dev_prelude::*;

pub use super::PublicKey;

pub type ListAllPublicKeysService = Arc<
    dyn for<'a> Service<
        &'a ListAllPublicKeys,
        Response = Vec<PublicKey>,
        Error = BoxedError,
    >,
>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAllPublicKeys;
