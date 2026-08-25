use base64::{self, Engine};
use serde::Deserialize;

// Question: Is key the public key? --> no
// What is used to sign the JWT?

// pub fn get_jwt_exp(raw_jwt: Option<String>, key: Uuid) -> Option<usize> {
//     let Some(token) = raw_jwt else {
//         return None;
//     };

//     // Verify the token (returns JWTClaims<NoCustomClaims>)
//     let decoded_claims = key
//         .verify_token::<NoCustomClaims>(&token, None)
//         .unwrap();

//     // Get the exp (expires_at) timestamp
//     let exp_epoch = decoded_claims.expires_at;

//     exp_epoch
// }

pub fn get_jwt_exp_no_decode(raw_jwt: Option<String>) -> Option<usize> {
    let Some(jwt) = raw_jwt else {
        return None;
    };

    let parts: Vec<&str> = jwt.split('.').collect();

    if parts.len() < 2 {
        // Err("Invalid JWT structure".into());
        return None;
    }

    // Decode the payload part (second element) using URL-safe base64 without padding
    let Ok(decoded_bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
    else {
        return None;
    };

    // Parse JSON payload
    let Ok(exp_epoch) = serde_json::from_slice(&decoded_bytes) else {
        return None;
    };

    exp_epoch
}
