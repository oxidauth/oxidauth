use std::{error::Error, fmt};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A freshly minted single-use state nonce, stored as `sha256(state)` — the
/// raw state never reaches the database (OXA-000010).
#[derive(Debug, Clone)]
pub struct CreateOAuthState {
    pub state_hash: Vec<u8>,
    pub client_key: Uuid,
    pub expires_at: DateTime<Utc>,
}

/// The consume request: the row is deleted iff it exists, was minted for
/// `client_key`, and has not expired. Single-use by construction.
#[derive(Debug, Clone)]
pub struct ConsumeOAuthState {
    pub state_hash: Vec<u8>,
    pub client_key: Uuid,
}

/// Raised when the callback carries a state this server did not recently
/// mint for this authority (OXA-000010).
///
/// Unknown, expired, and already-consumed states deliberately collapse into
/// `Invalid` — the storage cannot tell them apart and the endpoint must not
/// hand callers an oracle over the nonce space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OAuth2StateError {
    /// Shape violation (empty or overlong) — rejected before any hashing.
    Malformed,
    /// Not a live, unconsumed nonce minted for this authority.
    Invalid,
}

impl fmt::Display for OAuth2StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OAuth2StateError::*;

        match self {
            Malformed => write!(f, "oauth2 state is missing or malformed"),
            Invalid => write!(f, "oauth2 state is invalid or expired"),
        }
    }
}

impl Error for OAuth2StateError {
}
