use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod validate;

/// The only entitlement a TOTP authority puts on a jwt before its code has been
/// presented, and the only thing the refresh token minted alongside it can buy.
///
/// It lives here rather than beside the other system permission strings because
/// readers outside the server need it too: whoever mints the half session, whoever
/// has to keep it half (`refresh_tokens::exchange`), and whichever client must
/// recognise from the token alone that its session still owes a code all point at
/// this one string.
pub const TOTP_VALIDATE_PERMISSION: &str = "oxidauth:totp_code:validate";

#[derive(Debug, Serialize, Deserialize)]
pub struct TOTPValidationRes {
    pub jwt: String,
    pub refresh_token: Uuid,
}
