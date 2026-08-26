use base64::{self, Engine};
use jwt_simple::prelude::*;
use log::info;

use crate::public_keys::PublicKey;

// Question: Is key the public key? --> no
// What is used to sign the JWT?

pub fn get_jwt_exp(raw_jwt: &Option<String>, public_keys: &[PublicKey]) -> Result<usize, String> {
    let Some(token) = raw_jwt else {
        return Err("no jwt".to_string());
    };

    for key in public_keys.iter() {
        // Verify the token (returns JWTClaims<NoCustomClaims>)
        let public_key = match RS256PublicKey::from_pem(&key.public_key) {
            Ok(key) => key,
            Err(err) => {
                info!("error getting public key {}", err);

                continue;
            },
        };

        let claims = match public_key.verify_token::<NoCustomClaims>(&token, None) {
            Ok(claims) => claims,
            Err(err) => {
                info!("error verifying token {}", err);

                continue;
            },
        };

        // Get the exp (expires_at) timestamp
        let Some(Ok(exp_epoch)) = claims
            .expires_at
            .map(|exp| exp.as_secs().try_into())
        else {
            return Err("unable to fetch exp from jwt".to_string());
        };

        info!("this is the token expires at value {:?}", exp_epoch);
        // exp_epoch
        return Ok(exp_epoch);
    }

    return Err("unable to verify jwt for exp claim".to_string());
}
