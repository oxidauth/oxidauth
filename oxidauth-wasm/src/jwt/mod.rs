use base64::{self, Engine};
use log::info;

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
        info!("no jwt - probably not authed");
        return None;
    };

    info!("getting jwt parts");
    let parts: Vec<&str> = jwt.split('.').collect();

    if parts.len() < 2 {
        info!("invalid jwt length");
        return None;
    }

    // Decode the payload part (second element) using URL-safe base64 without padding
    let Ok(decoded_bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1])
    else {
        info!("unable to decode jwt bytes");
        return None;
    };

    // Parse JSON payload
    let Ok(exp_epoch) = serde_json::from_slice(&decoded_bytes) else {
        info!(
            "unable to serde_json decode exp_epoch from decoded jwt. Bytes example: {:?}",
            decoded_bytes
        );
        return None;
    };

    exp_epoch
}
