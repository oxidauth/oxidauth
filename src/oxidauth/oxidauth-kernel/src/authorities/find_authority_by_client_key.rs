use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::Authority;
use crate::error::BoxedError;

/// The lookup that answers "which authority does this client key open?".
///
/// It exists because the client key — not the strategy, not the name — is the
/// only handle a client app has on its authority: an instance hosts any number
/// of authorities sharing a strategy (`authorities.strategy` lost its UNIQUE
/// constraint in `20240523190325`), so `find_authority_by_strategy` answers with
/// an arbitrary one of them. Bootstrap is the caller that must not guess: the
/// authority it resolves is the one that gets `oxidauth:admin`.
///
/// Deliberately a service and not an HTTP route: a client app already holds its
/// own key and never needs to enumerate someone else's by key.
#[async_trait]
pub trait FindAuthorityByClientKeyServiceTrait: Send + Sync + 'static {
    async fn find_authority_by_client_key(
        &self,
        params: &FindAuthorityByClientKey,
    ) -> Result<Authority, BoxedError>;
}

pub type FindAuthorityByClientKeyService = Arc<dyn FindAuthorityByClientKeyServiceTrait>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAuthorityByClientKey {
    pub client_key: Uuid,
}
